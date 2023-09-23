use std::{sync::{Arc, RwLock, mpsc::{Receiver, Sender}}, net::SocketAddr};

use crate::{network::{connection_list::ConnectionList, Packet, Content, connection_request::ConnectionRequest}, config::config::Config, text::{text_list::TextList, text_request::TextRequest}, log::{logger::Logger, message_kind::MessageKind}};

pub struct Context
{
    pub movable: MovableContext,
    pub unmovable: UnmovableContext
}

pub struct MovableContext
{
    pub connection_list: Arc<RwLock<ConnectionList>>,
    pub text_list: Arc<RwLock<TextList>>,
    pub log: Logger,

    pub connection_requests_rx: Receiver<ConnectionRequest>,
    pub connection_requests_tx: Sender<ConnectionRequest>,

    pub text_requests_rx: Receiver<TextRequest>,
    pub text_requests_tx: Sender<TextRequest>,

    pub connection_queue_rx: Receiver<(Packet,SocketAddr)>,
    pub text_queue_rx: Receiver<(Packet,SocketAddr)>,
    pub voice_queue_rx: Receiver<(Packet,SocketAddr)>,
    pub sender_queue_rx: Receiver<(Content,SocketAddr)>,
    pub connection_queue_tx: Sender<(Packet,SocketAddr)>,
    pub text_queue_tx: Sender<(Packet,SocketAddr)>,
    pub voice_queue_tx: Sender<(Packet,SocketAddr)>,
    pub sender_queue_tx: Sender<(Content,SocketAddr)>,
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
        let log = Logger::new();
        let config = match config_path
        {
            Some(path) =>
            {
                match Config::from_file(path)
                {
                    Ok(c) => c,
                    Err(e) => {
                        log.log(
                            MessageKind::Error, 
                            &format!("Error occured while reading config file: {}",e))
                        .expect("The program is still singlethreaded, so this should never happen");
                        Config::default()
                    }
                }
            }
            None => {Config::default()}
        };
        
        let config = std::sync::Arc::new(std::sync::RwLock::new(config));

        let connection_list = Arc::new(RwLock::new(ConnectionList::new()));
        let text_list = Arc::new(RwLock::new(TextList::new()));

        let (connection_requests_tx, connection_requests_rx) = std::sync::mpsc::channel::<ConnectionRequest>();
        let (text_requests_tx, text_requests_rx) = std::sync::mpsc::channel::<TextRequest>();

        let (text_queue_tx, text_queue_rx) = std::sync::mpsc::channel::<(Packet,SocketAddr)>();
        let (connection_queue_tx, connection_queue_rx) = std::sync::mpsc::channel::<(Packet,SocketAddr)>();
        let (voice_queue_tx, voice_queue_rx) = std::sync::mpsc::channel::<(Packet,SocketAddr)>();
        let (sender_queue_tx, sender_queue_rx) = std::sync::mpsc::channel::<(Content,std::net::SocketAddr)>();
        let running = Arc::new(RwLock::new(true));
        Self
        {
            movable: MovableContext
            {
                connection_list,
                text_list,
                log,
                
                connection_requests_rx,
                connection_requests_tx,
                text_requests_rx,
                text_requests_tx,

                connection_queue_rx,
                text_queue_rx,
                voice_queue_rx,
                sender_queue_rx,
                connection_queue_tx,
                text_queue_tx,
                voice_queue_tx,
                sender_queue_tx,
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