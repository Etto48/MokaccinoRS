use std::{sync::{Arc, RwLock, mpsc::{Receiver, Sender}}, net::SocketAddr, thread::JoinHandle};

use crate::{network::{ConnectionList, Packet, Content}, log::Logger, config::Config, file::{threads::file, FileRequest}};

pub fn start(
    running: Arc<RwLock<bool>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    requests: Receiver<FileRequest>,
    file_queue: Receiver<(Packet,SocketAddr)>,
    sender_queue: Sender<(Content,SocketAddr)>,
    config: Arc<RwLock<Config>>
) -> Vec<JoinHandle<Result<(),String>>>
{
    let builder = std::thread::Builder::new().name("File".to_string());
    match builder.spawn(move || {
        file::run(running,
            connection_list, 
            log,
            requests,
            file_queue, 
            sender_queue,
            config)
    })
    {
        Ok(handle) => vec![handle],
        Err(e) => panic!("Error while creating thread File: {e}")
    }
}