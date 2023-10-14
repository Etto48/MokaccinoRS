use std::{net::{UdpSocket, SocketAddr}, sync::{Arc, mpsc::Sender, RwLock}};

use serializable::Serializable;

use crate::{network::{Packet, Content, ConnectionList, SecurePacket}, config::{Config, defines}, log::{Logger, MessageKind}};

pub fn run(
    running: Arc<RwLock<bool>>,
    socket: Arc<UdpSocket>,
    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    text_queue: Sender<(Packet,SocketAddr)>, 
    connection_queue: Sender<(Packet,SocketAddr)>,
    voice_queue: Sender<(Packet,SocketAddr)>,
    _config: Arc<RwLock<Config>>
) -> Result<(),String>
{
    let mut buffer = [0u8; defines::MAX_PACKET_SIZE];
    while *running.read().map_err(|e| e.to_string())?
    {
        match socket.recv_from(&mut buffer)
        {
            Ok((len,from)) => 
            {
                let (secure_packet,packet_size) = match SecurePacket::deserialize(&buffer[..len])
                {
                    Ok(sp) => sp,
                    Err(e) => 
                    {
                        log.log(MessageKind::Error, &format!("Error deserializing packet: {}",e))?;
                        continue;
                    }
                };
                if packet_size != len
                {
                    log.log(MessageKind::Error, &format!("Packet size mismatch: {} != {}",packet_size,len))?;
                    continue;
                }
                let packet = match secure_packet
                {
                    SecurePacket::Plaintext(p) => p,
                    SecurePacket::Ciphertext(c) => 
                    {
                        let connection_list = connection_list.read().map_err(|e|e.to_string())?;
                        if let Some(info) = connection_list.get_info_from_addr(&from)
                        {
                            match c.to_packet(&info.crypto_session_info.symmetric_key)
                            {
                                Ok(p) => p,
                                Err(e) => {
                                    log.log(MessageKind::Error, &format!("Error decrypting packet: {}",e))?;
                                    continue;
                                },
                            }
                        }
                        else
                        {
                            log.log(MessageKind::Error, &format!("Unknown user {} sent an encrypted message",from))?;
                            continue;
                        }
                        
                    }
                };
                //log.log(MessageKind::Event, &format!("Received {:?} from {}", packet, from))?;
                let queue = match &packet.content {
                    Content::Text(_,_) |
                    Content::AcknowledgeText(_,_) => {
                        &text_queue
                    },
                    Content::Ping |
                    Content::Pong |
                    Content::RequestConnection(_) |
                    Content::AcknowledgeConnection => 
                    {
                        &connection_queue
                    },
                    Content::Voice(_) => 
                    {
                        &voice_queue
                    },
                };
                queue.send((packet,from)).map_err(|e|e.to_string())?;
            },
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::TimedOut |
                    std::io::ErrorKind::WouldBlock => {}
                    e => {
                        log.log(MessageKind::Error, &format!("Error receiving packet: {}",e)).map_err(|e|e.to_string())?;
                    }
                }
            }
        }
    }
    Ok(())
}