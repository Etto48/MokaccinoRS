use std::{net::{UdpSocket, SocketAddr}, sync::{Arc, mpsc::Sender, RwLock}};

use crate::{network::{Packet, Serializable, Content, connection_list::ConnectionList}, config::config::Config};

pub fn run(
    running: Arc<RwLock<bool>>,
    socket: Arc<UdpSocket>,
    connection_list: Arc<RwLock<ConnectionList>>,
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
                        println!("Error deserializing packet: {}",e);
                        continue;
                    }
                };
                if packet_size != len
                {
                    println!("Packet size mismatch: {} != {}",packet_size,len);
                    continue;
                }
                println!("Received packet from: {from} ({packet_size}B) content: {packet:?}");
                let queue = match &packet.content {
                    Content::Text(_) => {
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
                    std::io::ErrorKind::TimedOut => {}
                    _ => {println!("Error receiving from socket");}
                }
            }
        }
    }
    Ok(())
}