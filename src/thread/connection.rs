use std::{thread::JoinHandle, sync::{mpsc::{Receiver, Sender}, Arc, RwLock}, net::SocketAddr};

use crate::{config::config::Config, network::{Packet, threads::connection, Content, connection_list::ConnectionList, connection_request::ConnectionRequest}, log::{logger::Logger}};

pub fn start(
    running: Arc<RwLock<bool>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    requests: Receiver<ConnectionRequest>,
    connection_queue: Receiver<(Packet,SocketAddr)>,
    sender_queue: Sender<(Content,SocketAddr)>,
    config: Arc<RwLock<Config>>
) -> Vec<JoinHandle<Result<(),String>>>
{
    let builder = std::thread::Builder::new().name("Connection".to_string());
    match builder.spawn(move || {
        connection::run(
            running, 
            connection_list, 
            log,
            requests,
            connection_queue,
            sender_queue,
            config)
    })
    {
        Ok(handle) => vec![handle],
        Err(e) => panic!("Error starting thread Connection: {e}")
    }   
}

#[cfg(test)]
mod tests
{
    use crate::{thread, network::ContactInfo, config::defines};
    use super::*;

    #[test]
    fn incoming_connection()
    {
        let context = thread::Context::new(None);
        let handles = start(
            context.unmovable.running.clone(),
            context.movable.connection_list.clone(),
            context.movable.log.clone(),
            context.movable.connection_requests_rx,
            context.movable.connection_queue_rx,
            context.movable.sender_queue_tx.clone(),
            context.unmovable.config.clone());
        assert_eq!(handles.len(),1);
        let remote_address = "0.0.0.0:4848".parse().unwrap();
        let remote_contact_info = ContactInfo::new("Test");
        context.movable.connection_queue_tx.send(
            (
                Packet::from_content_now(Content::RequestConnection(remote_contact_info.clone())),
                remote_address
            )
        ).unwrap();
        std::thread::sleep(defines::THREAD_QUEUE_TIMEOUT);
        if let Ok((content,dst)) = context.movable.sender_queue_rx.recv_timeout(defines::THREAD_QUEUE_TIMEOUT)
        {
            assert_eq!(content,
            Content::request_connection_from_config(&context.unmovable.config.read().unwrap()));
            assert_eq!(dst,remote_address);
        }
        else {
            panic!("Connection timed out");
        }
        context.movable.connection_queue_tx.send((
            Packet::from_content_now(Content::AcknowledgeConnection),
            remote_address
        )).unwrap();
        std::thread::sleep(defines::THREAD_QUEUE_TIMEOUT);
        {
            let connection_list = context.movable.connection_list.read().unwrap();
            if let Some(_info) = connection_list.get_info_from_name(remote_contact_info.name())
            {
                //success
            }
            else {
                println!("Connection list:");
                for (name,info) in connection_list.get_infos()
                {
                    println!("{}: {:?}",name,info);
                }
                panic!("Connection timed out");
            }
        }
    }
}