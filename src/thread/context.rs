use std::{sync::{Arc, RwLock, mpsc::{Receiver, Sender}}, net::SocketAddr};

use crate::{network::{connection_list::ConnectionList, Packet, Content}, config::config::Config};

pub struct Context
{
    pub movable: MovableContext,
    pub unmovable: UnmovableContext
}

pub struct MovableContext
{
    pub connection_list: Arc<RwLock<ConnectionList>>,
    pub connection_queue_rx: Receiver<(Packet,SocketAddr)>,
    pub text_queue_rx: Receiver<(Packet,SocketAddr)>,
    pub voice_queue_rx: Receiver<(Packet,SocketAddr)>,
    pub sender_queue_rx: Receiver<(Content,SocketAddr)>,
    pub connection_queue: Sender<(Packet,SocketAddr)>,
    pub text_queue: Sender<(Packet,SocketAddr)>,
    pub voice_queue: Sender<(Packet,SocketAddr)>,
    pub sender_queue: Sender<(Content,SocketAddr)>,
}

#[derive(Clone)]
pub struct UnmovableContext
{
    pub running: Arc<RwLock<bool>>,
    pub config: Arc<RwLock<Config>>,
}

impl Context
{
    pub fn new(config_path: Option<&str>) -> Self
    { 
        let config = match config_path
        {
            Some(path) =>
            {
                match Config::from_file(path)
                {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!(
                            "Error reading config:\n\
                            {}\n\
                            using default config:\n\
                            {}",e,toml::to_string(&Config::default()).expect("Default config is serializable"));
                        Config::default()
                    }
                }
            }
            None => {Config::default()}
        };
        
        let config = std::sync::Arc::new(std::sync::RwLock::new(config));

        let connection_list = Arc::new(RwLock::new(ConnectionList::new()));

        let (text_queue, text_queue_rx) = std::sync::mpsc::channel::<(Packet,SocketAddr)>();
        let (connection_queue, connection_queue_rx) = std::sync::mpsc::channel::<(Packet,SocketAddr)>();
        let (voice_queue, voice_queue_rx) = std::sync::mpsc::channel::<(Packet,SocketAddr)>();
        let (sender_queue, sender_queue_rx) = std::sync::mpsc::channel::<(Content,std::net::SocketAddr)>();
        let running = Arc::new(RwLock::new(true));
        Self
        {
            movable: MovableContext
            {
                connection_list,
                connection_queue_rx,
                text_queue_rx,
                voice_queue_rx,
                sender_queue_rx,
                connection_queue,
                text_queue,
                voice_queue,
                sender_queue,
            },
            unmovable: UnmovableContext
            {
                running,
                config,
            }
        }
    }
}

impl UnmovableContext
{
    pub fn stop(&self)
    {
        *self.running.write().expect("Error occurred while stopping threads (Poisoned RwLock)") = false;
    }
}