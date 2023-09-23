use std::{thread::JoinHandle, sync::{Arc, mpsc::{Sender, Receiver}, RwLock}, net::SocketAddr};

use crate::{network::{socket, threads::listener, threads::sender, Packet, Content, connection_list::ConnectionList}, config::config::Config, log::{logger::Logger}};

pub fn start(
    running: Arc<RwLock<bool>>,

    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,

    text_queue: Sender<(Packet,SocketAddr)>,
    connection_queue: Sender<(Packet,SocketAddr)>,
    voice_queue: Sender<(Packet,SocketAddr)>,

    sender_queue: Receiver<(Content,SocketAddr)>,

    config: Arc<RwLock<Config>>
) -> Vec<JoinHandle<Result<(),String>>>
{
    let listener_builder = std::thread::Builder::new().name("Listener".to_string());
    let sender_builder = std::thread::Builder::new().name("Sender".to_string());
    let socket = match socket::create(
        config.read().expect(
            "This RwLock should not be poisoned \
            as the program is still single threaded at this time").network.port)
    {
        Ok(socket) => socket,
        Err(e) => panic!("Error creating the socket {e}")
    };
    let socket = Arc::new(socket);
    let listener_socket = socket.clone();
    let sender_socket = socket.clone();
    let listener_config = config.clone();
    let sender_config = config.clone();
    let listener_connection_list = connection_list.clone();
    let sender_connection_list = connection_list.clone();
    let listener_running = running.clone();
    let sender_running = running.clone();
    let listener_log = log.clone();
    let sender_log = log.clone();
    let listener = match listener_builder.spawn(move || {
        listener::run(
            listener_running, 
            listener_socket, 
            listener_connection_list, 
            listener_log,
            text_queue, 
            connection_queue, 
            voice_queue, 
            listener_config)
    })
    {
        Ok(listener) => listener,
        Err(e) => panic!("Error starting thread Listener: {e}")
    };
    let sender = match sender_builder.spawn(move || {
        sender::run(
            sender_running,
            sender_socket, 
            sender_connection_list, 
            sender_log,
            sender_queue, 
            sender_config)
    })
    {
        Ok(sender) => sender,
        Err(e) => panic!("Error starting thread Sender: {e}")
    };
    vec![listener,sender]
}
