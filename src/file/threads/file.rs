use std::{sync::{Arc, RwLock, mpsc::{Receiver, Sender}}, net::SocketAddr};

use crate::{network::{ConnectionList, Packet, Content}, log::Logger, file::FileRequest, config::Config};

pub fn run(
    running: Arc<RwLock<bool>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    requests: Receiver<FileRequest>,
    file_queue: Receiver<(Packet,SocketAddr)>, 
    sender_queue: Sender<(Content,SocketAddr)>,
    config: Arc<RwLock<Config>>
) -> Result<(),String>
{
    Ok(())
}