use std::{thread::JoinHandle, sync::{mpsc::Receiver, Arc, RwLock}, net::SocketAddr};

use crate::{network::{Packet, connection_list::ConnectionList}, config::config::Config, text::threads::text};

pub fn start(
    running: Arc<RwLock<bool>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    text_queue: Receiver<(Packet,SocketAddr)>,
    config: Arc<RwLock<Config>>
) -> Vec<JoinHandle<Result<(),String>>>
{
    let builder = std::thread::Builder::new().name("Text".to_string());
    match builder.spawn(move || {
        text::run(running, 
            connection_list, 
            text_queue, 
            config)
    })
    {
        Ok(handle) => vec![handle],
        Err(e) => panic!("Error while creating thread Text: {e}")
    }
}