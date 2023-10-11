use std::{net::{UdpSocket, SocketAddr}, sync::{Arc, mpsc::Receiver, RwLock}};

use serializable::Serializable;

use crate::{network::{Packet, Content, ConnectionList, SecurePacket}, config::{Config, defines}, log::Logger, crypto::Ciphertext};

pub fn run(
    running: Arc<RwLock<bool>>,
    socket: Arc<UdpSocket>, 
    connection_list: Arc<RwLock<ConnectionList>>,
    _log: Logger,
    queue: Receiver<(Content,SocketAddr)>, 
    _config: Arc<RwLock<Config>>) -> Result<(),String>
{
    while *running.read().map_err(|e|e.to_string())?
    {
        match queue.recv_timeout(defines::THREAD_QUEUE_TIMEOUT)
        {
            Ok((content, dst)) =>
            {
                //log.log(MessageKind::Event, &format!("Sending {:?} to {}",content, dst))?;
                let packet = Packet::from_content_now(content);
                let secure_packet = 
                {
                    let connection_list = connection_list.read().map_err(|e|e.to_string())?;
                    if let Some(info) = connection_list.get_info_from_addr(&dst)
                    {
                        SecurePacket::Ciphertext(Ciphertext::from_packet(packet, &info.crypto_session_info.symmetric_key))
                    }
                    else
                    {
                        SecurePacket::Plaintext(packet)    
                    }
                };
                
                let bytes = secure_packet.serialize();
                socket.send_to(&bytes, dst).map_err(|e|e.to_string())?;
            }
            Err(e) =>
            {
                match e
                {
                    std::sync::mpsc::RecvTimeoutError::Timeout => {},
                    std::sync::mpsc::RecvTimeoutError::Disconnected => 
                    {
                        return if !*running.read().map_err(|e|e.to_string())?
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