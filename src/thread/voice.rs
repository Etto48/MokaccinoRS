use std::{thread::JoinHandle, sync::{mpsc::{Receiver, Sender}, Arc, RwLock}, net::SocketAddr};

use crate::{voice::threads::voice, network::{Packet, Content, connection_list::ConnectionList}, config::config::Config, log::{logger::Logger}};

pub fn start(
    running: Arc<RwLock<bool>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    voice_queue: Receiver<(Packet,SocketAddr)>, 
    sender_queue: Sender<(Content,SocketAddr)>,
    config: Arc<RwLock<Config>>) -> Vec<JoinHandle<Result<(),String>>>
{
    let builder = std::thread::Builder::new().name("Voice".to_string());
    match builder.spawn(move || {
        voice::run(
            running, 
            connection_list, 
            log,
            voice_queue, 
            sender_queue, 
            config)
    })
    {
        Ok(handle) => vec![handle],
        Err(e) => panic!("Error creating thread Voice: {e}")
    }
}