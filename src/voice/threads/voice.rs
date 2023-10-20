use std::{sync::{mpsc::{Receiver, Sender, RecvTimeoutError}, Arc, RwLock, Mutex, MutexGuard}, net::SocketAddr, collections::VecDeque, time::Instant};

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
    config: Arc<RwLock<Config>>
)
{
    let host = cpal::default_host();
    let mut context = get_voice_context(&host, &config, &log).unwrap();

    let mut recently_ended: Option<(SocketAddr, Instant)> = None;
    context.input_stream.pause().unwrap();
    context.output_stream.pause().unwrap();

    while *running.read().unwrap()
    {
        match voice_queue.recv_timeout(defines::THREAD_QUEUE_TIMEOUT) {
            Ok((packet,from)) =>
            {
                match packet.content
                {
                    Content::Voice(voice) =>
                    {
                        if let Some(interlocutor_address) = *voice_interlocutor.lock().unwrap()
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
                                        let mut output_channel = context.output_channel.lock().unwrap();
                                        output_channel.extend(context.output_resampler_buffer[0].iter().take(output_frames));
                                    }
                                    else {
                                        log.log(MessageKind::Error, &format!("Failed to resample voice packet from {}", from)).unwrap();
                                    }
                                }
                                else {
                                    log.log(MessageKind::Error, &format!("Failed to decode voice packet from {}", from)).unwrap();
                                }
                            }
                            else 
                            {
                                // someone is calling while a call is already in progress
                                sender_queue.send((Content::EndVoice, from)).unwrap();
                            }
                        }
                        else {
                            if let Some(from_name) = connection_list.read().unwrap().get_name(&from)
                            {
                                let mut ignore = false;
                                if let Some((addr, _time)) = recently_ended
                                {
                                    if addr == from
                                    {
                                        ignore = true;
                                        sender_queue.send((Content::EndVoice, from)).unwrap();
                                    }
                                }
                                if !ignore
                                {
                                    ui_notifications.send(UiNotification::IncomingCall(from_name.to_string())).unwrap();
                                }
                            }
                            else 
                            {
                                log.log(MessageKind::Error, &format!("Received voice packet from unknown peer {}", from)).unwrap();
                                sender_queue.send((Content::EndVoice, from)).unwrap();
                            }
                        }
                    },
                    Content::EndVoice =>
                    {
                        let voice_interlocutor = voice_interlocutor.lock().unwrap();
                        if let Some(interlocutor_address) = *voice_interlocutor
                        {
                            if interlocutor_address == from
                            {
                                stop_transmission_no_lock(
                                    voice_interlocutor, 
                                    &context.input_channel, 
                                    &context.output_channel, 
                                    &context.input_stream, 
                                    &context.output_stream,
                                    &log).unwrap();
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
                        let voice_interlocutor = voice_interlocutor.lock().unwrap();
                        if let Some(interlocutor_address) = *voice_interlocutor
                        {
                            let connection_list = connection_list.read().unwrap();
                            if connection_list.get_name(&interlocutor_address).is_none()
                            {
                                stop_transmission_no_lock(
                                    voice_interlocutor, 
                                    &context.input_channel, 
                                    &context.output_channel, 
                                    &context.input_stream, 
                                    &context.output_stream,
                                    &log).unwrap();
                            }
                        }
                        if let Some((_addr, time)) = recently_ended
                        {
                            if time.elapsed() > defines::VOICE_ENDED_TIMEOUT
                            {
                                recently_ended = None;
                            }
                        }
                    },
                    RecvTimeoutError::Disconnected => 
                    {
                        if !*running.read().unwrap()
                        {return} 
                        else 
                        {panic!("Voice channel broken")}
                    }
                }
            },
        }
        {
            let config = config.read().unwrap();
            let voice_gain = config.voice.gain.clamp(defines::MIN_GAIN, defines::MAX_GAIN);
            context.decoder.set_gain(voice_gain).unwrap();
        }
        if let Some(interlocutor_addres) = *voice_interlocutor.lock().unwrap()
        {
            let mut input_channel = context.input_channel.lock().unwrap();
            let needed_frames = context.input_resampler.input_frames_next();
            if input_channel.len() >= needed_frames
            {
                let data = input_channel.drain(..needed_frames).collect::<Vec<f32>>();
                
                let (_input_frames, output_frames) = 
                context.input_resampler.process_into_buffer(
                        &[data], 
                        &mut context.input_resampler_buffer, 
                        None
                    ).unwrap();
                
                let encoded = context.encoder.encode_vec_float(&context.input_resampler_buffer[0][..output_frames], defines::VOICE_MAX_TRANSMISSION_SIZE).unwrap();
                let content = Content::Voice(encoded);
                sender_queue.send((content, interlocutor_addres)).unwrap();
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
                            &log).unwrap();
                    },
                    VoiceRequest::StopTransmission(target_address) =>
                    {
                        let voice_interlocutor = voice_interlocutor.lock().unwrap();
                        if let Some(interlocutor_address) = *voice_interlocutor
                        {
                            if interlocutor_address == target_address
                            {
                                stop_transmission_no_lock(
                                    voice_interlocutor, 
                                    &context.input_channel, 
                                    &context.output_channel, 
                                    &context.input_stream, 
                                    &context.output_stream,
                                    &log).unwrap();
                            }
                        }
                        sender_queue.send((Content::EndVoice, target_address)).unwrap();
                        recently_ended = Some((target_address, Instant::now()));
                    },
                    VoiceRequest::ReloadConfiguration => 
                    {
                        let old_interlocutor = stop_transmission(
                            &voice_interlocutor, 
                            &context.input_channel, 
                            &context.output_channel, 
                            &context.input_stream, 
                            &context.output_stream, 
                            &log).unwrap();
                        context = get_voice_context(&host, &config, &log).unwrap();
                        if let Some(addr) = old_interlocutor
                        {
                            start_transmission(
                                &voice_interlocutor, 
                                addr, 
                                &context.input_stream, 
                                &context.output_stream,
                                &context.input_channel,
                                &context.output_channel,
                                &log).unwrap();
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
                        if !*running.read().unwrap()
                        {return} 
                        else 
                        {panic!("Voice channel broken")}
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
        &log).unwrap();
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
        log.log(MessageKind::Event, &format!("Voice chat with {} ended", addr)).unwrap();
    }
    let ret = *interlocutor;
    *interlocutor = None;
    input_stream.pause().unwrap();
    output_stream.pause().unwrap();
    input_channel.lock().unwrap().clear();
    output_channel.lock().unwrap().clear();
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
    let interlocutor = interlocutor.lock().unwrap();
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
    let mut interlocutor = interlocutor.lock().unwrap();
    *interlocutor = Some(target_address);
    input_channel.lock().unwrap().clear();
    output_channel.lock().unwrap().clear();
    input_stream.play().unwrap();
    output_stream.play().unwrap();
    log.log(MessageKind::Event, &format!("Voice chat with {} started", target_address)).unwrap();
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
    let config = config.read().unwrap();
    let input_device = 
    if let Some(input_device_name) = config.voice.input_device.clone()
    {
        let input_devices = host.input_devices().unwrap();
        let mut input_device = None;
        for device in input_devices
        {
            if device.name().unwrap() == input_device_name
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
            log.log(MessageKind::Error, &format!("Input device {} not found", input_device_name)).unwrap();
            log.log(MessageKind::Event, "Using default input device").unwrap();
            let input_devices_string = host.input_devices().unwrap().map(|device|device.name().map_err(|e|e.to_string())).collect::<Result<Vec<String>,String>>().unwrap().join(", ");
            log.log(MessageKind::Event, &format!("Available input devices: {}",input_devices_string)).unwrap();
            get_default_input_device(&host).unwrap()
        }
    }
    else 
    {
        get_default_input_device(&host).unwrap()
    };
    let output_device = 
    if let Some(output_device_name) = config.voice.output_device.clone()
    {
        let output_devices = host.output_devices().unwrap();
        let mut output_device = None;
        for device in output_devices
        {
            if device.name().unwrap() == output_device_name
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
            log.log(MessageKind::Error, &format!("Output device {} not found", output_device_name)).unwrap();
            log.log(MessageKind::Event, "Using default output device").unwrap();
            let input_devices_string = host.output_devices().unwrap().map(|device|device.name().map_err(|e|e.to_string())).collect::<Result<Vec<String>,String>>().unwrap().join(", ");
            log.log(MessageKind::Event, &format!("Available output devices: {}",input_devices_string)).unwrap();
            get_default_output_device(&host).unwrap()
        }
    }
    else
    {
        get_default_output_device(&host).unwrap()
    };
    Ok((input_device, output_device))
}

fn get_voice_context(host: &Host, config: &Arc<RwLock<Config>>, log: &Logger) -> Result<VoiceContext,String>
{
    let (input_device, output_device) = get_io_devices(&host, config, &log).unwrap();
    let input_configs = input_device.default_input_config().unwrap();
    let output_configs = output_device.default_output_config().unwrap();
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
    ).unwrap();

    let input_resampler_buffer = input_resampler.output_buffer_allocate(true);

    let output_resampler = rubato::FftFixedIn::<f32>::new(
        defines::VOICE_TRANSMISSION_SAMPLE_RATE,
        output_config.sample_rate.0 as usize,
        defines::VOICE_BUFFER_SIZE,
        1,
        1
    ).unwrap();

    let output_resampler_buffer = output_resampler.output_buffer_allocate(true);

    let mut encoder = opus::Encoder::new(
        defines::VOICE_TRANSMISSION_SAMPLE_RATE as u32,
        opus::Channels::Mono,
        opus::Application::Voip
    ).unwrap();

    encoder.set_bitrate(defines::VOICE_TRANSMISSION_BITRATE).unwrap();

    let decoder = opus::Decoder::new(
        defines::VOICE_TRANSMISSION_SAMPLE_RATE as u32,
        opus::Channels::Mono
    ).unwrap();

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
    },None).unwrap();

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
    },None).unwrap();


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