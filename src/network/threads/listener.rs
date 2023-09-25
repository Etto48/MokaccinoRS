use std::{net::{UdpSocket, SocketAddr}, sync::{Arc, mpsc::Sender, RwLock}};

use crate::{network::{Packet, Serializable, Content, ConnectionList}, config::Config, log::{Logger, MessageKind}};

pub fn run(
    running: Arc<RwLock<bool>>,
    socket: Arc<UdpSocket>,
    _connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    text_queue: Sender<(Packet,SocketAddr)>, 
    connection_queue: Sender<(Packet,SocketAddr)>,
    voice_queue: Sender<(Packet,SocketAddr)>,
    _config: Arc<RwLock<Config>>
) -> Result<(),String>
{
    let mut buffer = [0u8; 1024];
    while running.read().map_err(|e| e.to_string())?.clone()
    {
        match socket.recv_from(&mut buffer)
        {
            Ok((len,from)) => 
            {
                let (packet,packet_size) = match Packet::deserialize(&buffer[..len])
                {
                    Ok(p) => p,
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
                    Content::VoiceRequest |
                    Content::VoiceAccept |
                    Content::VoiceAcknowledge |
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
                    e => {println!("Error receiving from socket: {e}");}
                }
            }
        }
    }
    Ok(())
}