use std::{sync::{mpsc::{Receiver, Sender, RecvTimeoutError}, Arc, RwLock, Mutex}, net::SocketAddr, collections::VecDeque};

use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use rubato::Resampler;

use crate::{network::{Packet, Content, ConnectionList}, config::{Config, defines}, log::{Logger, MessageKind}, voice::VoiceRequest};

pub fn run(
    running: Arc<RwLock<bool>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    requests: Receiver<VoiceRequest>,
    voice_interlocutor: Arc<Mutex<Option<SocketAddr>>>,
    voice_queue: Receiver<(Packet,SocketAddr)>, 
    sender_queue: Sender<(Content,SocketAddr)>,
    _config: Arc<RwLock<Config>>) -> Result<(),String>
{
    let host = cpal::default_host();
    //let devices = host.devices().map_err(|e|e.to_string())?;
    let input_device = match host.default_input_device()
    {
        Some(device) => device,
        None => return Err("No input device found".to_string()),
    };
    let output_device = match host.default_output_device()
    {
        Some(device) => device,
        None => return Err("No output device found".to_string()),
    };
    let input_configs = input_device.default_input_config().map_err(|e|e.to_string())?;
    let output_configs = output_device.default_output_config().map_err(|e|e.to_string())?;
    let input_config = input_configs.config();
    let output_config = output_configs.config();

    println!("Chanels: Input: {}, Output: {}", input_config.channels, output_config.channels);

    let input_channel = Arc::new(Mutex::new(VecDeque::<f32>::new()));
    let moved_input_channel = input_channel.clone();
    let output_channel = Arc::new(Mutex::new(VecDeque::<f32>::new()));
    let moved_output_channel = output_channel.clone();

    let mut input_resampler = rubato::FftFixedOut::<f32>::new(
        input_config.sample_rate.0 as usize,
        defines::VOICE_TRANSMISSION_SAMPLE_RATE,
        defines::VOICE_BUFFER_SIZE,
        1,
        1
    ).map_err(|e|e.to_string())?;

    let mut input_resampler_buffer = input_resampler.output_buffer_allocate(true);

    let mut output_resampler = rubato::FftFixedIn::<f32>::new(
        defines::VOICE_TRANSMISSION_SAMPLE_RATE,
        output_config.sample_rate.0 as usize,
        defines::VOICE_BUFFER_SIZE,
        1,
        1
    ).map_err(|e|e.to_string())?;

    let mut output_resampler_buffer = output_resampler.output_buffer_allocate(true);

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

    input_stream.pause().map_err(|e|e.to_string())?;
    output_stream.pause().map_err(|e|e.to_string())?;

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
                                if let Ok((_input_frames, output_frames)) = output_resampler.process_into_buffer(
                                    &[voice], 
                                    &mut output_resampler_buffer, 
                                    None) 
                                {
                                    let mut output_channel = output_channel.lock().unwrap();
                                    output_channel.extend(output_resampler_buffer[0].iter().take(output_frames));
                                }
                            }
                        }
                        else {
                            log.log(MessageKind::Error, &format!("Received voice packet from {} without active voice chat", from))?;
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
                        if let Some(interlocutor_address) = *voice_interlocutor.lock().map_err(|e|e.to_string())?
                        {
                            let connection_list = connection_list.read().map_err(|e|e.to_string())?;
                            if connection_list.get_name(&interlocutor_address).is_none()
                            {
                                stop_transmission(
                                    &voice_interlocutor, 
                                    &input_channel, 
                                    &output_channel, 
                                    &input_stream, 
                                    &output_stream,
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
        if let Some(interlocutor_addres) = *voice_interlocutor.lock().map_err(|e|e.to_string())?
        {
            let mut input_channel = input_channel.lock().unwrap();
            let needed_frames = input_resampler.input_frames_next();
            if input_channel.len() >= needed_frames
            {
                let data = input_channel.drain(..needed_frames).collect::<Vec<f32>>();
                
                let (_input_frames, _output_frames) = 
                    input_resampler.process_into_buffer(
                        &[data], 
                        &mut input_resampler_buffer, 
                        None
                    ).unwrap();
                
                let content = Content::Voice(input_resampler_buffer[0].clone());
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
                            &input_stream, 
                            &output_stream,
                            &input_channel,
                            &output_channel,
                            &log)?;
                    },
                    VoiceRequest::StopTransmission =>
                    {
                        stop_transmission(
                            &voice_interlocutor, 
                            &input_channel, 
                            &output_channel, 
                            &input_stream, 
                            &output_stream,
                            &log)?;
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
    Ok(())
}

fn stop_transmission(
    interlocutor: &Arc<Mutex<Option<SocketAddr>>>,
    input_channel: &Arc<Mutex<VecDeque<f32>>>,
    output_channel: &Arc<Mutex<VecDeque<f32>>>,
    input_stream: &impl StreamTrait,
    output_stream: &impl StreamTrait,
    log: &Logger
) -> Result<(),String>
{
    let mut interlocutor = interlocutor.lock().map_err(|e|e.to_string())?;
    if let Some(addr) = *interlocutor
    {
        log.log(MessageKind::Event, &format!("Voice chat with {} ended", addr))?;
    }
    *interlocutor = None;
    input_stream.pause().unwrap();
    output_stream.pause().unwrap();
    input_channel.lock().unwrap().clear();
    output_channel.lock().unwrap().clear();
    Ok(())
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
    input_channel.lock().unwrap().clear();
    output_channel.lock().unwrap().clear();
    input_stream.play().unwrap();
    output_stream.play().unwrap();
    log.log(MessageKind::Event, &format!("Voice chat with {} started", target_address))?;
    Ok(())
}