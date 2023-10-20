use std::{sync::{Arc, RwLock, mpsc::{Receiver, Sender}}, net::SocketAddr};

use crate::{network::{ConnectionList, Packet, Content}, log::Logger, file::FileRequest, config::{Config, defines}};

pub fn run(
    running: Arc<RwLock<bool>>,
    _connection_list: Arc<RwLock<ConnectionList>>,
    _log: Logger,
    _requests: Receiver<FileRequest>,
    file_queue: Receiver<(Packet,SocketAddr)>, 
    _sender_queue: Sender<(Content,SocketAddr)>,
    _config: Arc<RwLock<Config>>
)
{
    while *running.read().unwrap()
    {
        match file_queue.recv_timeout(defines::THREAD_QUEUE_TIMEOUT)
        {
            Ok((_packet, _from)) => 
            {
                todo!();
            },
            Err(e) => 
            {
                match e
                {
                    std::sync::mpsc::RecvTimeoutError::Timeout => 
                    {

                    },
                    std::sync::mpsc::RecvTimeoutError::Disconnected => 
                    {
                        if !*running.read().unwrap()
                        {return} 
                        else 
                        {panic!("File channel broken")}
                    },
                }
            },
        }
    }
}