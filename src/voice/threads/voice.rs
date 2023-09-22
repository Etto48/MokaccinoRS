use std::{sync::{mpsc::{Receiver, Sender, RecvTimeoutError}, Arc, RwLock}, net::SocketAddr};

use crate::{network::{Packet, Content, connection_list::ConnectionList}, config::{config::Config, defines}, log::logger::Logger};

pub fn run(
    running: Arc<RwLock<bool>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    voice_queue: Receiver<(Packet,SocketAddr)>, 
    sender_queue: Sender<(Content,SocketAddr)>,
    config: Arc<RwLock<Config>>) -> Result<(),String>
{
    while running.read().map_err(|e|e.to_string())?.clone()
    {
        match voice_queue.recv_timeout(defines::THREAD_QUEUE_TIMEOUT) {
            Ok((packet,from)) =>
            {
                todo!("Handle voice packet");
            }
            Err(e) =>
            {
                match e
                {
                    RecvTimeoutError::Timeout => {},
                    RecvTimeoutError::Disconnected => 
                    {
                        return if !running.read().map_err(|e|e.to_string())?.clone()
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