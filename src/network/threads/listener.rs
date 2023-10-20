use std::{net::{UdpSocket, SocketAddr}, sync::{Arc, mpsc::Sender, RwLock}};

use serializable::Serializable;

use crate::{network::{Packet, Content, ConnectionList, SecurePacket}, config::{Config, defines}, log::{Logger, MessageKind}};

pub fn run(
    running: Arc<RwLock<bool>>,
    socket: Arc<UdpSocket>,
    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    text_queue: Sender<(Packet,SocketAddr)>, 
    file_queue: Sender<(Packet,SocketAddr)>,
    connection_queue: Sender<(Packet,SocketAddr)>,
    voice_queue: Sender<(Packet,SocketAddr)>,
    _config: Arc<RwLock<Config>>
)
{
    let mut buffer = [0u8; defines::MAX_PACKET_SIZE];
    while *running.read().unwrap()
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
                        log.log(MessageKind::Error, &format!("Error deserializing packet: {}",e)).unwrap();
                        continue;
                    }
                };
                if packet_size != len
                {
                    log.log(MessageKind::Error, &format!("Packet size mismatch: {} != {}",packet_size,len)).unwrap();
                    continue;
                }
                let packet = match secure_packet
                {
                    SecurePacket::Plaintext(p) => 
                    {

                        match p.content 
                        {
                            Content::RequestConnection(_) |
                            Content::AcknowledgeConnection => {},
                            _  => {
                                log.log(MessageKind::Error, &format!("Received unexpected plaintext packet from {}",from)).unwrap();
                            }
                        };
                        p
                    },
                    SecurePacket::Ciphertext(c) => 
                    {
                        let connection_list = connection_list.read().unwrap();
                        if let Some(info) = connection_list.get_info_from_addr(&from)
                        {
                            match c.to_packet(&info.crypto_session_info.symmetric_key)
                            {
                                Ok(p) => p,
                                Err(e) => {
                                    log.log(MessageKind::Error, &format!("Error decrypting packet: {}",e)).unwrap();
                                    continue;
                                },
                            }
                        }
                        else
                        {
                            log.log(MessageKind::Error, &format!("Unknown user {} sent an encrypted message",from)).unwrap();
                            continue;
                        }
                        
                    }
                };
                //log.log(MessageKind::Event, &format!("Received {:.unwrap()} from {}", packet, from)).unwrap();
                let queue = match &packet.content {
                    Content::Text(_,_) |
                    Content::AcknowledgeText(_,_) => {
                        &text_queue
                    },
                    Content::Ping |
                    Content::Pong |
                    Content::RequestConnection(_) |
                    Content::AcknowledgeConnection |
                    Content::RequestUserInfo(_,_) |
                    Content::UserInfo(_) => 
                    {
                        &connection_queue
                    },
                    Content::Voice(_) |
                    Content::EndVoice => 
                    {
                        &voice_queue
                    },
                };
                queue.send((packet,from)).unwrap();
            },
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::TimedOut |
                    std::io::ErrorKind::WouldBlock => {}
                    e => {
                        log.log(MessageKind::Error, &format!("Error receiving packet: {}",e)).unwrap();
                    }
                }
            }
        }
    }
}