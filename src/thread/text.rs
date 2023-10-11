use std::{thread::JoinHandle, sync::{mpsc::{Receiver, Sender}, Arc, RwLock}, net::SocketAddr};

use crate::{network::{Packet, ConnectionList, Content}, config::Config, text::{threads::text, TextList, TextRequest}, log::Logger};

pub fn start(
    running: Arc<RwLock<bool>>,
    text_list: Arc<RwLock<TextList>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    requests: Receiver<TextRequest>,
    text_queue: Receiver<(Packet,SocketAddr)>,
    sender_queue: Sender<(Content,SocketAddr)>,
    config: Arc<RwLock<Config>>
) -> Vec<JoinHandle<Result<(),String>>>
{
    let builder = std::thread::Builder::new().name("Text".to_string());
    match builder.spawn(move || {
        text::run(running, 
            text_list,
            connection_list, 
            log,
            requests,
            text_queue, 
            sender_queue,
            config)
    })
    {
        Ok(handle) => vec![handle],
        Err(e) => panic!("Error while creating thread Text: {e}")
    }
}

#[cfg(test)]
mod tests {
    use crate::config::defines;
    use crate::crypto::SymmetricKey;
    use crate::thread::Context;

    use super::*;
    use std::thread;

    #[test]
    fn send_text() {
        let context = Context::new(None);

        let handles = start(
            context.unmovable.running.clone(),
            context.movable.text_list.clone(),
            context.movable.connection_list.clone(),
            context.movable.log.clone(),
            context.movable.text_requests_rx,
            context.movable.text_queue_rx,
            context.movable.sender_queue_tx.clone(),
            context.unmovable.config.clone(),
        );

        // Send a text request
        let request = TextRequest{
            text: "TestText".to_string(),
            dst: "TEST".to_string(),
        };
        let symmetric_key = SymmetricKey::random();
        context.movable.connection_list.write().unwrap().add("TEST", "127.0.0.1:4848".parse().unwrap(), symmetric_key);
        context.movable.text_requests_tx.send(request).unwrap();

        // Wait for the request to be processed
        thread::sleep(defines::THREAD_QUEUE_TIMEOUT);

        // Check the sender queue for the packet
        if let Ok((content,dst)) = context.movable.sender_queue_rx.recv_timeout(defines::THREAD_QUEUE_TIMEOUT) {
            if let Content::Text(text,_nonce) = content
            {
                assert_eq!(text, "TestText".to_string());
                assert_eq!(dst, "127.0.0.1:4848".parse().unwrap());
            }
            else {
                panic!("Wrong content type");
            }
        }
        else {
            panic!("No packet was sent to the sender queue");
        }        

        // Stop the server
        context.unmovable.stop();

        // Wait for the server to stop
        for handle in handles {
            handle.join().unwrap().unwrap();
        }
    }
}