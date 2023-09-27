use std::{sync::{mpsc::{Receiver, Sender}, Arc, RwLock}, net::SocketAddr, collections::HashMap, time::{Instant, Duration}};

use rand::RngCore;

use crate::{network::{Packet, ConnectionList, Content}, config::{Config, defines}, text::{TextList, TextInfo, TextDirection, TextRequest}, log::Logger};

pub fn run(
    running: Arc<RwLock<bool>>,
    text_list: Arc<RwLock<TextList>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    _log: Logger,
    requests: Receiver<TextRequest>,
    text_queue: Receiver<(Packet,SocketAddr)>, 
    sender_queue: Sender<(Content,SocketAddr)>,
    config: Arc<RwLock<Config>>) -> Result<(),String>
{
    let mut pending_messages = HashMap::<(String,String,u64),Instant>::new();
    while *running.read().map_err(|e|e.to_string())?
    {
        match text_queue.recv_timeout(defines::THREAD_QUEUE_TIMEOUT) {
            Ok((packet,from)) => 
            {
                match packet.content {
                    Content::Text(text,nonce) => 
                    {
                        // check if the user is in the connection list
                        let connection_list = connection_list.read().map_err(|e|e.to_string())?;
                        if let Some(name) = connection_list.get_name(&from)
                        {
                            // check if the text was already received
                            let mut text_list = text_list.write().map_err(|e|e.to_string())?;
                            let info = TextInfo {
                                text: text.clone(),
                                nonce,
                                direction: TextDirection::Incoming,
                            };
                            // add to the list if new
                            text_list.add(name, info);
                            // send ack
                            let content = Content::AcknowledgeText(text,nonce);
                            sender_queue.send((content,from)).map_err(|e|e.to_string())?;
                        }
                    },
                    Content::AcknowledgeText(text,nonce) =>
                    {
                        let connection_list = connection_list.read().map_err(|e|e.to_string())?;
                        if let Some(name) = connection_list.get_name(&from)
                        {
                            text_list.write().map_err(|e|e.to_string())?.add(name, 
                                TextInfo { 
                                    text: text.clone(), 
                                    nonce,
                                    direction: TextDirection::Outgoing });
                            pending_messages.remove(&(name.to_string(),text,nonce));
                        }
                    }
                    _ => unreachable!("Text thread received non-text packet: {:?}",packet)
                }
            },
            Err(e) => 
            {
                match e
                {
                    std::sync::mpsc::RecvTimeoutError::Timeout => {
                        let config = config.read().map_err(|e|e.to_string())?.clone();
                        let mut to_remove = Vec::new();
                        for ((name,message,nonce),last_seen) in pending_messages.iter_mut()
                        {
                            if last_seen.elapsed() >= Duration::from_millis(config.network.timeout_ms)
                            {
                                // check if the user is still connected
                                let connection_list = connection_list.read().map_err(|e|e.to_string())?;
                                if let Some(addr) = connection_list.get_address(name)
                                {
                                    // resend the message
                                    let content = Content::Text(message.clone(),*nonce);
                                    sender_queue.send((content,*addr)).map_err(|e|e.to_string())?;
                                    // update the last seen time
                                    *last_seen = Instant::now();
                                }
                                else {
                                    // remove the message
                                    to_remove.push((name.to_string(),message.clone(),*nonce));
                                }
                            }
                        }
                        for (name,message,nonce) in to_remove
                        {
                            pending_messages.remove(&(name,message,nonce));
                        }
                    },
                    std::sync::mpsc::RecvTimeoutError::Disconnected => 
                    {
                        return if !*running.read().map_err(|e|e.to_string())?
                        {Ok(())} 
                        else 
                        {Err("Text channel broken".to_string())}
                    }
                }
            },
        }
        // check if there are any new messages to send
        match requests.try_recv()
        {
            Ok(TextRequest{text, dst}) => 
            {
                let connection_list = connection_list.read().map_err(|e|e.to_string())?;
                if let Some(addr) = connection_list.get_address(&dst)
                {
                    let nonce = gen_nonce();
                    let content = Content::Text(text.clone(),nonce);
                    sender_queue.send((content,*addr)).map_err(|e|e.to_string())?;
                    pending_messages.insert((dst,text,nonce),Instant::now());
                }
            },
            Err(e) => 
            {
                match e
                {
                    std::sync::mpsc::TryRecvError::Empty => {},
                    std::sync::mpsc::TryRecvError::Disconnected => 
                    {
                        return if !*running.read().map_err(|e|e.to_string())?
                        {Ok(())} 
                        else 
                        {Err("Text channel broken".to_string())}
                    },
                }
            },
        }
    }
    Ok(())
}

fn gen_nonce() -> u64
{
    rand::thread_rng().next_u64()
}