use std::{thread::JoinHandle, sync::{mpsc::{Receiver, Sender}, Arc, RwLock}, net::SocketAddr};

use crate::{network::{Packet, ConnectionList, Content}, config::Config, text::{threads::text, TextList, TextRequest}, log::Logger};

pub fn start(
    running: Arc<RwLock<bool>>,
    text_list: Arc<RwLock<TextList>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    requests: Receiver<TextRequest>,
    text_queue: Receiver<(Packet,SocketAddr)>,
    sender_queue: Sender<(Content,SocketAddr)>,
    config: Arc<RwLock<Config>>
) -> Vec<JoinHandle<Result<(),String>>>
{
    let builder = std::thread::Builder::new().name("Text".to_string());
    match builder.spawn(move || {
        text::run(running, 
            text_list,
            connection_list, 
            log,
            requests,
            text_queue, 
            sender_queue,
            config)
    })
    {
        Ok(handle) => vec![handle],
        Err(e) => panic!("Error while creating thread Text: {e}")
    }
}