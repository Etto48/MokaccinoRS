use std::{sync::{mpsc::Receiver, Arc, RwLock}, net::SocketAddr};

use crate::{network::{Packet, connection_list::ConnectionList}, config::{config::Config, defines}};

pub fn run(
    running: Arc<RwLock<bool>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    text_queue: Receiver<(Packet,SocketAddr)>, 
    config: Arc<RwLock<Config>>) -> Result<(),String>
{
    while running.read().map_err(|e|e.to_string())?.clone()
    {
        match text_queue.recv_timeout(defines::THREAD_QUEUE_TIMEOUT) {
            Ok((packet,from)) => 
            {

            },
            Err(e) => 
            {
                match e
                {
                    std::sync::mpsc::RecvTimeoutError::Timeout => {},
                    std::sync::mpsc::RecvTimeoutError::Disconnected => 
                    {
                        return if !running.read().map_err(|e|e.to_string())?.clone()
                        {Ok(())} 
                        else 
                        {Err("Text channel broken".to_string())}
                    }
                }
            },
        }
    }
    Ok(())
}