use std::{sync::{mpsc::{Sender, Receiver}, Arc, RwLock}, net::SocketAddr, collections::HashMap, time::{Duration, Instant}};

use crate::{config::{Config, defines}, network::{ConnectionList, Packet, Content, ContactInfo, ConnectionRequest, LastingContactInfo}, log::{Logger, MessageKind}, crypto::{CryptoHandshakeInfo, PrivateKey}};

pub fn run(
    running: Arc<RwLock<bool>>,
    connection_list: Arc<RwLock<ConnectionList>>,
    log: Logger,
    requests: Receiver<ConnectionRequest>,
    connection_queue: Receiver<(Packet,SocketAddr)>, 
    sender_queue: Sender<(Content,SocketAddr)>,
    config: Arc<RwLock<Config>>
) -> Result<(),String>
{
    let mut pending_requests = HashMap::<SocketAddr,(Option<ContactInfo>,CryptoHandshakeInfo,Instant,u16)>::new();
    while *running.read().map_err(|e|e.to_string())?
    {
        match connection_queue.recv_timeout(defines::THREAD_QUEUE_TIMEOUT)
        {
            Ok((packet,from)) => 
            {
                match &packet.content
                {
                    Content::Ping => 
                    {
                        // lock less resources at the same time possible
                        let mut is_connected = false;
                        {
                            let mut connection_list = connection_list.write().map_err(|e|e.to_string())?;
                            if let Some(info) = connection_list.get_info_from_addr_mut(&from)
                            {
                                info.reset_strikes();
                                is_connected = true;
                            }
                        }
                        if is_connected
                        {
                            sender_queue.send((Content::Pong, from)).map_err(|e|e.to_string())?;
                        }
                    },
                    Content::Pong => 
                    {
                        {
                            let mut connection_list = connection_list.write().map_err(|e|e.to_string())?;
                            if let Some(info) = connection_list.get_info_from_addr_mut(&from)
                            {
                                info.reset_strikes();
                            }
                        }
                    },
                    Content::RequestConnection(signed_contact_info) => 
                    {
                        if let Ok(unsafe_info) = signed_contact_info.info()
                        {
                            let public_key = {
                                let mut config = config.write().map_err(|e|e.to_string())?;
                                if let Some(known_host) = config.network.known_hosts.get(unsafe_info.name())
                                {
                                    known_host.crypto_info().public_key.clone()
                                }
                                else
                                {
                                    config.network.known_hosts.insert(unsafe_info.name().to_owned(), LastingContactInfo::new(unsafe_info.name(), &unsafe_info.crypto_info().into_lasting()));
                                    unsafe_info.crypto_info().public_key.clone()
                                }
                            };
                            if let Ok(contact_info) = &signed_contact_info.into_contact_info(&public_key)
                            {
                                {
                                    let connection_list = connection_list.read().map_err(|e|e.to_string())?;
                                    if let Some(name) = connection_list.get_name(&from)
                                    {
                                        if name == contact_info.name()
                                        {
                                            // the other peer is already connected, maybe the ack sent from this peer was lost, send another ack
                                            sender_queue.send((Content::AcknowledgeConnection, from)).map_err(|e|e.to_string())?;
                                        }
                                    }
                                }
                                if let Some((option_info, crypto_handshake_info, _last_seen, _strikes)) = pending_requests.get_mut(&from)
                                {
                                    // this peer started the connection and was expecting a response from the other peer
                                    match option_info
                                    {
                                        Some(saved_contact_info) =>
                                        {
                                            // the other peer sent a duplicate request connection packet, perhaps our response was lost, repeat the response
                                            if saved_contact_info == contact_info
                                            {
                                                // no info has changed, send the same response
                                                let config_reader = config.read().map_err(|e|e.to_string())?;
                                                sender_queue.send((Content::request_connection_from_config(&config_reader,crypto_handshake_info.local_ecdhe_key.public_key().clone()), from)).map_err(|e|e.to_string())?;
                                            }
                                        },
                                        None =>
                                        {
                                            // this peer sent the initial request and the other peer responded, finalize the connection
                                            let success = 
                                            {
                                                let mut connection_list = connection_list.write().map_err(|e|e.to_string())?;
                                                crypto_handshake_info.remote_ecdhe_key = Some(contact_info.crypto_info().public_key.clone());
                                                if let Ok(symmetric_key) = crypto_handshake_info.derive()
                                                {
                                                    connection_list.add(contact_info.name(), from, symmetric_key);
                                                    true
                                                }
                                                else 
                                                {
                                                    false
                                                }
                                            };
                                            if success
                                            {
                                                log.log(MessageKind::Event, &format!("Connection to {} established", contact_info.name()))?;
                                                sender_queue.send((Content::AcknowledgeConnection, from)).map_err(|e|e.to_string())?;
                                            }
                                            else {
                                                log.log(MessageKind::Error, &format!("Key exchange with {} failed", contact_info.name()))?;
                                            }
                                            pending_requests.remove(&from);
                                            
                                        }
                                    }
                                }
                                else 
                                {
                                    // the other peer is requesting the connection
                                    let config_reader = config.read().map_err(|e|e.to_string())?;
                                    let mut accept_connection = true;
                                    if let Some(whitelist) = &config_reader.network.whitelist
                                    {
                                        accept_connection = whitelist.iter().any(|name| name == contact_info.name())
                                    }
                                    if accept_connection
                                    {
                                        // accept the connection
                                        let private_ecdhe_key = PrivateKey::new();
                                        let public_ecdhe_key = private_ecdhe_key.public_key();
                                        let crypto_handshake_info = CryptoHandshakeInfo {
                                            local_ecdhe_key: private_ecdhe_key,
                                            remote_ecdhe_key: Some(contact_info.crypto_info().ecdhe_public_key.clone()),
                                        };
                                        sender_queue.send((Content::request_connection_from_config(&config_reader, public_ecdhe_key), from)).map_err(|e|e.to_string())?;
                                        pending_requests.insert(from, (Some(contact_info.clone()),crypto_handshake_info,Instant::now(),0));
                                    }
                                }
                            }
                            else
                            {
                                log.log(MessageKind::Error, &format!("Invalid signature from {}", from))?;
                            }
                        }
                    },
                    Content::AcknowledgeConnection => 
                    {
                        if let Some((option_info, crypto_handshake_info, _last_seen, _strikes)) = pending_requests.get_mut(&from)
                        {
                            if let Some(contact_info) = option_info
                            {
                                let mut connection_ok = false;
                                // the other peer started the connection and has acknowledged our response
                                {
                                    let mut connection_list = connection_list.write().map_err(|e|e.to_string())?;
                                    if let Ok(symmetric_key) = crypto_handshake_info.derive()
                                    {
                                        connection_list.add(contact_info.name(), from, symmetric_key);
                                        connection_ok = true;
                                    }
                                }
                                if connection_ok
                                {
                                    log.log(MessageKind::Event, &format!("Connection to {} established", contact_info.name()))?;
                                }
                                else
                                {
                                    log.log(MessageKind::Error, &format!("Connection to {} failed", contact_info.name()))?;
                                }
                                pending_requests.remove(&from);
                            }
                            else {
                                // this peer started the connection and was not expecting an ack from the other peer
                                // this should not happen but the program should not crash
                            }
                        }
                    },
                    _ => unreachable!("Connection thread received non-connection packet: {:?}",packet)
                }
            },
            Err(e) => 
            {
                match e
                {
                    std::sync::mpsc::RecvTimeoutError::Timeout => {
                        //check for timed out pending requests
                        let config = config.read().map_err(|e|e.to_string())?.clone();

                        let mut timed_out_pending_requests = Vec::new();
                        {
                            for (address, (_info, crypto_handshake_info, last_seen, strikes)) in &mut pending_requests
                            {
                                if last_seen.elapsed() > Duration::from_millis(config.network.timeout_ms)
                                {
                                    if *strikes >= config.network.timeout_strikes
                                    {
                                        // remove the connection from the pending requests
                                        timed_out_pending_requests.push(address.clone());
                                    }
                                    else
                                    {
                                        // send another request
                                        sender_queue.send((Content::request_connection_from_config(&config, crypto_handshake_info.local_ecdhe_key.public_key()), address.clone())).map_err(|e|e.to_string())?;
                                        *strikes += 1;
                                        *last_seen = Instant::now();
                                    }
                                }
                            }
                        }
                        for address in timed_out_pending_requests
                        {
                            pending_requests.remove(&address);
                            log.log(MessageKind::Error, &format!("Connection to {} timed out", address))?;
                        }

                        // check for timed out connections
                        let mut timed_out_connections = Vec::new();
                        {
                            let mut connection_list = connection_list.write().map_err(|e|e.to_string())?;
                            for (address, info) in connection_list.get_infos()
                            {
                                if info.last_seen.elapsed() > Duration::from_millis(config.network.ping_ms) ||
                                (info.strikes > 0 && info.last_seen.elapsed() > Duration::from_millis(config.network.timeout_ms))
                                {
                                    if info.strikes >= config.network.timeout_strikes
                                    {
                                        // remove the connection from the connection list
                                        timed_out_connections.push(address.clone());
                                    }
                                    else
                                    {
                                        // resend ping
                                        sender_queue.send((Content::Ping, address.clone())).map_err(|e|e.to_string())?;
                                        let info = connection_list.get_info_from_addr_mut(&address).expect("just got it with write lock");
                                        info.add_strike();
                                    }
                                }
                            }
                            for address in timed_out_connections
                            {
                                connection_list.remove_with_address(&address);
                                log.log(MessageKind::Event, &format!("Connection to {} timed out", address))?;
                            }
                        }
                    },
                    std::sync::mpsc::RecvTimeoutError::Disconnected => 
                    {
                        return if !*running.read().map_err(|e|e.to_string())?
                        {Ok(())} 
                        else 
                        {Err("Connection channel broken".to_string())}
                    }
                }
            },
        }
        // check if there are any new connection-related requests
        match requests.try_recv()  
        {
            Ok(request) => 
            {
                match request
                {
                    ConnectionRequest::Connect(to) => 
                    {
                        let config = config.read().map_err(|e|e.to_string())?.clone();
                        let private_ecdhe_key = PrivateKey::new();
                        let public_ecdhe_key = private_ecdhe_key.public_key();
                        let crypto_handshake_info = CryptoHandshakeInfo
                        {
                            local_ecdhe_key: private_ecdhe_key,
                            remote_ecdhe_key: None,
                        };
                        sender_queue.send((Content::request_connection_from_config(&config, public_ecdhe_key), to)).map_err(|e|e.to_string())?;
                        pending_requests.insert(to, (None,crypto_handshake_info,Instant::now(),0));
                    },
                    ConnectionRequest::Disconnect(from) => 
                    {
                        let mut connection_list = connection_list.write().map_err(|e|e.to_string())?;
                        connection_list.remove_with_name(&from);
                    },
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
                        {Err("Connection channel broken".to_string())}
                    },
                }
            },
        }
    }
    Ok(())
}