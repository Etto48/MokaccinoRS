use std::{sync::{mpsc::{Receiver, Sender, RecvTimeoutError}, Arc, RwLock, Mutex, MutexGuard}, net::SocketAddr, collections::VecDeque};

use cpal::{traits::{HostTrait, DeviceTrait, StreamTrait}, Device, Host};
use rubato::Resampler;

use crate::{network::{Packet, Content, ConnectionList}, config::{Config, defines}, log::{Logger, MessageKind}, voice::{VoiceRequest, VoiceContext}, ui::UiNotification};

pub fn run(
    running: Arc<RwLock<bool>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    requests: Receiver<VoiceRequest>,
    voice_interlocutor: Arc<Mutex<Option<SocketAddr>>>,
    ui_notifications: Sender<UiNotification>,
    voice_queue: Receiver<(Packet,SocketAddr)>, 
    sender_queue: Sender<(Content,SocketAddr)>,
    config: Arc<RwLock<Config>>) -> Result<(),String>
{
    let host = cpal::default_host();
    let mut context = get_voice_context(&host, &config, &log)?;

    context.input_stream.pause().map_err(|e|e.to_string())?;
    context.output_stream.pause().map_err(|e|e.to_string())?;

    while *running.read().map_err(|e|e.to_string())?
    {
        match voice_queue.recv_timeout(defines::THREAD_QUEUE_TIMEOUT) {
            Ok((packet,from)) =>
            {
                match packet.content
                {
                    Content::Voice(voice) =>
                    {
                        if let Some(interlocutor_address) = *voice_interlocutor.lock().map_err(|e|e.to_string())?
                        {
                            if interlocutor_address == from
                            {
                                let mut decoded = vec![0.0;defines::VOICE_BUFFER_SIZE];
                                if let Ok(decoded_len) = context.decoder.decode_float(&voice, &mut decoded, false)
                                {
                                    decoded.truncate(decoded_len);
                                    if let Ok((_input_frames, output_frames)) = context.output_resampler.process_into_buffer(
                                        &[decoded], 
                                        &mut context.output_resampler_buffer, 
                                        None) 
                                    {
                                        let mut output_channel = context.output_channel.lock().map_err(|e|e.to_string())?;
                                        output_channel.extend(context.output_resampler_buffer[0].iter().take(output_frames));
                                    }
                                    else {
                                        log.log(MessageKind::Error, &format!("Failed to resample voice packet from {}", from))?;
                                    }
                                }
                                else {
                                    log.log(MessageKind::Error, &format!("Failed to decode voice packet from {}", from))?;
                                }
                            }
                        }
                        else {
                            if let Some(from_name) = connection_list.read().map_err(|e|e.to_string())?.get_name(&from)
                            {
                                ui_notifications.send(UiNotification::IncomingCall(from_name.to_string())).map_err(|e|e.to_string())?;
                            }
                            else 
                            {
                                log.log(MessageKind::Error, &format!("Received voice packet from unknown peer {}", from))?;
                            }
                        }
                    },
                    _ => {},
                }
            }
            Err(e) =>
            {
                match e
                {
                    RecvTimeoutError::Timeout => {
                        let voice_interlocutor = voice_interlocutor.lock().map_err(|e|e.to_string())?;
                        if let Some(interlocutor_address) = *voice_interlocutor
                        {
                            let connection_list = connection_list.read().map_err(|e|e.to_string())?;
                            if connection_list.get_name(&interlocutor_address).is_none()
                            {
                                stop_transmission_no_lock(
                                    voice_interlocutor, 
                                    &context.input_channel, 
                                    &context.output_channel, 
                                    &context.input_stream, 
                                    &context.output_stream,
                                    &log)?;
                            }
                        }
                    },
                    RecvTimeoutError::Disconnected => 
                    {
                        return if !*running.read().map_err(|e|e.to_string())?
                        {Ok(())} 
                        else 
                        {Err("Voice channel broken".to_string())}
                    }
                }
            },
        }
        {
            let config = config.read().map_err(|e|e.to_string())?;
            let voice_gain = config.voice.gain.clamp(defines::MIN_GAIN, defines::MAX_GAIN);
            context.decoder.set_gain(voice_gain).map_err(|e|e.to_string())?;
        }
        if let Some(interlocutor_addres) = *voice_interlocutor.lock().map_err(|e|e.to_string())?
        {
            let mut input_channel = context.input_channel.lock().map_err(|e|e.to_string())?;
            let needed_frames = context.input_resampler.input_frames_next();
            if input_channel.len() >= needed_frames
            {
                let data = input_channel.drain(..needed_frames).collect::<Vec<f32>>();
                
                let (_input_frames, output_frames) = 
                context.input_resampler.process_into_buffer(
                        &[data], 
                        &mut context.input_resampler_buffer, 
                        None
                    ).map_err(|e|e.to_string())?;
                
                let encoded = context.encoder.encode_vec_float(&context.input_resampler_buffer[0][..output_frames], defines::VOICE_MAX_TRANSMISSION_SIZE).map_err(|e|e.to_string())?;
                let content = Content::Voice(encoded);
                sender_queue.send((content, interlocutor_addres)).map_err(|e|e.to_string())?;
            }
        }
        match requests.try_recv()
        {
            Ok(request) =>
            {
                match request
                {
                    VoiceRequest::StartTransmission(target_address) =>
                    {
                        start_transmission(
                            &voice_interlocutor, 
                            target_address, 
                            &context.input_stream, 
                            &context.output_stream,
                            &context.input_channel,
                            &context.output_channel,
                            &log)?;
                    },
                    VoiceRequest::StopTransmission =>
                    {
                        stop_transmission(
                            &voice_interlocutor, 
                            &context.input_channel, 
                            &context.output_channel, 
                            &context.input_stream, 
                            &context.output_stream,
                            &log)?;
                    },
                    VoiceRequest::ReloadConfiguration => 
                    {
                        let old_interlocutor = stop_transmission(
                            &voice_interlocutor, 
                            &context.input_channel, 
                            &context.output_channel, 
                            &context.input_stream, 
                            &context.output_stream, 
                            &log)?;
                        context = get_voice_context(&host, &config, &log)?;
                        if let Some(addr) = old_interlocutor
                        {
                            start_transmission(
                                &voice_interlocutor, 
                                addr, 
                                &context.input_stream, 
                                &context.output_stream,
                                &context.input_channel,
                                &context.output_channel,
                                &log)?;
                        }
                    },
                }
            },
            Err(e) =>
            {
                match e
                {
                    std::sync::mpsc::TryRecvError::Empty => {},
                    std::sync::mpsc::TryRecvError::Disconnected => 
                    {
                        return if !*running.read().map_err(|e|e.to_string())?
                        {Ok(())} 
                        else 
                        {Err("Voice channel broken".to_string())}
                    }
                }
            },
        }
    }
    stop_transmission(
        &voice_interlocutor, 
        &context.input_channel, 
        &context.output_channel, 
        &context.input_stream, 
        &context.output_stream, 
        &log).map_err(|e|e.to_string())?;
    Ok(())
}

fn stop_transmission_no_lock(
    mut interlocutor: MutexGuard<Option<SocketAddr>>,
    input_channel: &Arc<Mutex<VecDeque<f32>>>,
    output_channel: &Arc<Mutex<VecDeque<f32>>>,
    input_stream: &impl StreamTrait,
    output_stream: &impl StreamTrait,
    log: &Logger
) -> Result<Option<SocketAddr>,String>
{
    if let Some(addr) = *interlocutor
    {
        log.log(MessageKind::Event, &format!("Voice chat with {} ended", addr))?;
    }
    let ret = *interlocutor;
    *interlocutor = None;
    input_stream.pause().map_err(|e|e.to_string())?;
    output_stream.pause().map_err(|e|e.to_string())?;
    input_channel.lock().map_err(|e|e.to_string())?.clear();
    output_channel.lock().map_err(|e|e.to_string())?.clear();
    Ok(ret)
}

fn stop_transmission(
    interlocutor: &Arc<Mutex<Option<SocketAddr>>>,
    input_channel: &Arc<Mutex<VecDeque<f32>>>,
    output_channel: &Arc<Mutex<VecDeque<f32>>>,
    input_stream: &impl StreamTrait,
    output_stream: &impl StreamTrait,
    log: &Logger
) -> Result<Option<SocketAddr>,String>
{
    let interlocutor = interlocutor.lock().map_err(|e|e.to_string())?;
    stop_transmission_no_lock(
        interlocutor,
        input_channel,
        output_channel,
        input_stream,
        output_stream,
        log)
}

fn start_transmission(
    interlocutor: &Arc<Mutex<Option<SocketAddr>>>,
    target_address: SocketAddr,
    input_stream: &impl StreamTrait,
    output_stream: &impl StreamTrait,
    input_channel: &Arc<Mutex<VecDeque<f32>>>,
    output_channel: &Arc<Mutex<VecDeque<f32>>>,
    log: &Logger
) -> Result<(),String>
{
    let mut interlocutor = interlocutor.lock().map_err(|e|e.to_string())?;
    *interlocutor = Some(target_address);
    input_channel.lock().map_err(|e|e.to_string())?.clear();
    output_channel.lock().map_err(|e|e.to_string())?.clear();
    input_stream.play().map_err(|e|e.to_string())?;
    output_stream.play().map_err(|e|e.to_string())?;
    log.log(MessageKind::Event, &format!("Voice chat with {} started", target_address))?;
    Ok(())
}

fn get_default_input_device(host: &Host) -> Result<Device,String>
{
    match host.default_input_device()
    {
        Some(device) => Ok(device),
        None => Err("No input device found".to_string()),
    }
}

fn get_default_output_device(host: &Host) -> Result<Device,String>
{
    match host.default_output_device()
    {
        Some(device) => Ok(device),
        None => Err("No output device found".to_string()),
    }
}

fn get_io_devices(host: &Host, config: &Arc<RwLock<Config>>, log: &Logger) -> Result<(Device,Device),String>
{
    let config = config.read().map_err(|e|e.to_string())?;
    let input_device = 
    if let Some(input_device_name) = config.voice.input_device.clone()
    {
        let input_devices = host.input_devices().map_err(|e|e.to_string())?;
        let mut input_device = None;
        for device in input_devices
        {
            if device.name().map_err(|e|e.to_string())? == input_device_name
            {
                input_device = Some(device);
            }
        }
        if let Some(input_device) = input_device
        {
            input_device
        }
        else
        {
            log.log(MessageKind::Error, &format!("Input device {} not found", input_device_name))?;
            log.log(MessageKind::Event, "Using default input device")?;
            let input_devices_string = host.input_devices().map_err(|e|e.to_string())?.map(|device|device.name().map_err(|e|e.to_string())).collect::<Result<Vec<String>,String>>()?.join(", ");
            log.log(MessageKind::Event, &format!("Available input devices: {}",input_devices_string))?;
            get_default_input_device(&host)?
        }
    }
    else 
    {
        get_default_input_device(&host)?
    };
    let output_device = 
    if let Some(output_device_name) = config.voice.output_device.clone()
    {
        let output_devices = host.output_devices().map_err(|e|e.to_string())?;
        let mut output_device = None;
        for device in output_devices
        {
            if device.name().map_err(|e|e.to_string())? == output_device_name
            {
                output_device = Some(device);
            }
        }
        if let Some(output_device) = output_device
        {
            output_device
        }
        else
        {
            log.log(MessageKind::Error, &format!("Output device {} not found", output_device_name))?;
            log.log(MessageKind::Event, "Using default output device")?;
            let input_devices_string = host.output_devices().map_err(|e|e.to_string())?.map(|device|device.name().map_err(|e|e.to_string())).collect::<Result<Vec<String>,String>>()?.join(", ");
            log.log(MessageKind::Event, &format!("Available output devices: {}",input_devices_string))?;
            get_default_output_device(&host)?
        }
    }
    else
    {
        get_default_output_device(&host)?
    };
    Ok((input_device, output_device))
}

fn get_voice_context(host: &Host, config: &Arc<RwLock<Config>>, log: &Logger) -> Result<VoiceContext,String>
{
    let (input_device, output_device) = get_io_devices(&host, config, &log)?;
    let input_configs = input_device.default_input_config().map_err(|e|e.to_string())?;
    let output_configs = output_device.default_output_config().map_err(|e|e.to_string())?;
    let input_config = input_configs.config();
    let output_config = output_configs.config();
    let input_channel = Arc::new(Mutex::new(VecDeque::<f32>::new()));
    let moved_input_channel = input_channel.clone();
    let output_channel = Arc::new(Mutex::new(VecDeque::<f32>::new()));
    let moved_output_channel = output_channel.clone();

    let input_resampler = rubato::FftFixedOut::<f32>::new(
        input_config.sample_rate.0 as usize,
        defines::VOICE_TRANSMISSION_SAMPLE_RATE,
        defines::VOICE_BUFFER_SIZE,
        1,
        1
    ).map_err(|e|e.to_string())?;

    let input_resampler_buffer = input_resampler.output_buffer_allocate(true);

    let output_resampler = rubato::FftFixedIn::<f32>::new(
        defines::VOICE_TRANSMISSION_SAMPLE_RATE,
        output_config.sample_rate.0 as usize,
        defines::VOICE_BUFFER_SIZE,
        1,
        1
    ).map_err(|e|e.to_string())?;

    let output_resampler_buffer = output_resampler.output_buffer_allocate(true);

    let mut encoder = opus::Encoder::new(
        defines::VOICE_TRANSMISSION_SAMPLE_RATE as u32,
        opus::Channels::Mono,
        opus::Application::Voip
    ).map_err(|e|e.to_string())?;

    encoder.set_bitrate(defines::VOICE_TRANSMISSION_BITRATE).map_err(|e|e.to_string())?;

    let decoder = opus::Decoder::new(
        defines::VOICE_TRANSMISSION_SAMPLE_RATE as u32,
        opus::Channels::Mono
    ).map_err(|e|e.to_string())?;

    let input_channels = input_config.channels as usize;
    let output_channels = output_config.channels as usize;

    let input_stream = input_device.build_input_stream(&input_config, move |data: &[f32], _| {
        let mut input_channel = moved_input_channel.lock().unwrap();
        input_channel.reserve(data.len()/2);
        let mut channel = 0;
        for sample in data.iter()
        {
            if channel == 0
            {
                input_channel.push_back(*sample);
            }
            channel += 1;
            channel %= input_channels;
        }
    }, move |_err| {
        todo!("Handle input error");
    },None).map_err(|e|e.to_string())?;

    let output_stream = output_device.build_output_stream(&output_config, move |data: &mut [f32], _| {
        let mut output_channel = moved_output_channel.lock().unwrap();
        for sample in data.chunks_exact_mut(output_channels)
        {
            let s = match output_channel.pop_front()
            {
                Some(sample) => sample,
                None => 0.0,
            };
            for channel in sample.iter_mut()
            {
                *channel = s;
            }
        }
    }, move |_err| {
        todo!("Handle output error");
    },None).map_err(|e|e.to_string())?;


    Ok(VoiceContext
    {
        input_stream,
        output_stream,
        decoder,
        encoder,
        input_resampler,
        output_resampler,
        input_resampler_buffer,
        output_resampler_buffer,
        input_channel,
        output_channel,
    })
}