use std::{thread::JoinHandle, sync::{mpsc::{Receiver, Sender}, Arc, RwLock, Mutex}, net::SocketAddr};

use crate::{voice::{threads::voice, VoiceRequest}, network::{Packet, Content, ConnectionList}, config::Config, log::Logger, ui::ui_notification::UiNotification};

pub fn start(
    running: Arc<RwLock<bool>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    requests: Receiver<VoiceRequest>,
    voice_interlocutor: Arc<Mutex<Option<SocketAddr>>>,
    ui_notifications: Sender<UiNotification>,
    voice_queue: Receiver<(Packet,SocketAddr)>, 
    sender_queue: Sender<(Content,SocketAddr)>,
    config: Arc<RwLock<Config>>) -> Vec<JoinHandle<()>>
{
    let builder = std::thread::Builder::new().name("Voice".to_string());
    match builder.spawn(move || {
        voice::run(
            running, 
            connection_list, 
            log,
            requests,
            voice_interlocutor,
            ui_notifications,
            voice_queue, 
            sender_queue, 
            config)
    })
    {
        Ok(handle) => vec![handle],
        Err(e) => panic!("Error creating thread Voice: {e}")
    }
}