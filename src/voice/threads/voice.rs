use std::{sync::{mpsc::{Receiver, Sender, RecvTimeoutError}, Arc, RwLock}, net::SocketAddr};

use crate::{network::{Packet, Content, connection_list::ConnectionList}, config::{config::Config, defines}, log::logger::Logger};

pub fn run(
    running: Arc<RwLock<bool>>,
    _connection_list: Arc<RwLock<ConnectionList>>,
    _log: Logger,
    voice_queue: Receiver<(Packet,SocketAddr)>, 
    _sender_queue: Sender<(Content,SocketAddr)>,
    _config: Arc<RwLock<Config>>) -> Result<(),String>
{
    while running.read().map_err(|e|e.to_string())?.clone()
    {
        match voice_queue.recv_timeout(defines::THREAD_QUEUE_TIMEOUT) {
            Ok((_packet,_from)) =>
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