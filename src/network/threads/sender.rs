use std::{net::{UdpSocket, SocketAddr}, sync::{Arc, mpsc::Receiver, RwLock}};

use crate::{network::{Packet, Serializable, Content, connection_list::ConnectionList}, config::{config::Config, defines}, log::{logger::Logger, message_kind::MessageKind}};

pub fn run(
    running: Arc<RwLock<bool>>,
    socket: Arc<UdpSocket>, 
    _connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    queue: Receiver<(Content,SocketAddr)>, 
    config: Arc<RwLock<Config>>) -> Result<(),String>
{
    while running.read().map_err(|e|e.to_string())?.clone()
    {
        match queue.recv_timeout(defines::THREAD_QUEUE_TIMEOUT)
        {
            Ok((content, dst)) =>
            {
                log.log(MessageKind::Event, &format!("Sending {:?} to {}",content, dst))?;
                let packet = Packet::from_content_now(content);
                let bytes = packet.serialize();
                socket.send_to(&bytes, dst).map_err(|e|e.to_string())?;
            }
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
                        {Err("Sender channel broken".to_string())}
                    }
                }
            },
        }
    }
    Ok(())
}