use std::{sync::{mpsc::{Sender, Receiver}, Arc, RwLock}, net::SocketAddr, collections::HashMap, time::{Duration, Instant}};

use crate::{config::{Config, defines}, network::{ConnectionList, Packet, Content, ContactInfo, ConnectionRequest, LastingContactInfo, UserInfo}, log::{Logger, MessageKind}, crypto::{CryptoHandshakeInfo, PrivateKey, CryptoLastingInfo}};

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
    let mut pending_user_info_requests = HashMap::<(String,SocketAddr),(Option<SocketAddr>,Instant)>::new();
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
                            let (public_key, mut add_to_known_hosts) = {
                                let config = config.read().map_err(|e|e.to_string())?;
                                if let Some(known_host) = config.network.known_hosts.get(unsafe_info.name())
                                {
                                    (known_host.crypto_info().public_key.clone(),false)
                                }
                                else
                                {
                                    log.log(MessageKind::Event, &format!("New peer {} added",unsafe_info.name()))?;
                                    (unsafe_info.crypto_info().public_key.clone(),true)
                                }
                            };
                            if public_key != unsafe_info.crypto_info().public_key
                            {
                                log.log(MessageKind::Error, &format!("Public key mismatch for {}",unsafe_info.name()))?;
                                continue;
                            }
                            match &signed_contact_info.into_contact_info(&public_key)
                            {
                                Ok(contact_info) =>
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
                                                    crypto_handshake_info.remote_ecdhe_key = Some(contact_info.crypto_info().ecdhe_public_key.clone());
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
                                        else {
                                            add_to_known_hosts = false;
                                        }
                                    }
                                    if add_to_known_hosts
                                    {
                                        let mut config = config.write().map_err(|e|e.to_string())?;
                                        config.network.known_hosts.insert(unsafe_info.name().to_string(),LastingContactInfo::new(
                                            unsafe_info.name(),
                                            &unsafe_info.crypto_info().into_lasting(),
                                        ));
                                    }
                                }
                                Err(e) => 
                                {
                                    log.log(MessageKind::Error, &format!("Invalid signature from {}: {}", from, e))?;
                                }
                            };
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
                    Content::RequestUserInfo(name,ttl) =>
                    {
                        let ttl = std::cmp::min(ttl,&defines::MAX_FIND_TTL);
                        let self_info = 
                        {
                            let config = config.read().map_err(|e|e.to_string())?;
                            if config.network.name == *name
                            {
                                Some(UserInfo::self_from_config(&config))
                            }
                            else 
                            {
                                None
                            }
                        };
                        if let Some(info) = self_info
                        {
                            sender_queue.send((Content::UserInfo(info), from)).map_err(|e|e.to_string())?;
                        }
                        else
                        {
                            let user_info =
                            {
                                let config = config.read().map_err(|e|e.to_string())?.clone();
                                let connection_list = connection_list.read().map_err(|e|e.to_string())?;
                                UserInfo::new(name, &connection_list, &config)
                            };
                            if user_info.address().is_none() && *ttl > 0
                            { // the user is not connected
                                find_user(Some(from), name, ttl - 1, &mut pending_user_info_requests, connection_list.clone(), &sender_queue)?;
                            }
                            else
                            {
                                sender_queue.send((Content::UserInfo(user_info), from)).map_err(|e|e.to_string())?;
                            }
                        }
                    },
                    Content::UserInfo(info) =>
                    {
                        let mut done = false;
                        if let Some((prev, _time)) = pending_user_info_requests.get(&(info.name().to_string(),from))
                        {
                            if let Some(public_key) = info.public_key()
                            {
                                {
                                    let mut config = config.write().map_err(|e|e.to_string())?;
                                    let known_info = config.network.known_hosts.entry(info.name().to_string());
                                    known_info.or_insert(LastingContactInfo::new(info.name(), &CryptoLastingInfo::new(public_key)));
                                }
                                if let Some(address) = info.address()
                                {
                                    if prev.is_none()
                                    {
                                        // we are the source of the request
                                        request_connection(address, &mut pending_requests, &sender_queue, config.clone())?;
                                    }
                                }
                                if let Some(addr) = prev
                                {
                                    // we need to forward the response
                                    sender_queue.send((Content::UserInfo(info.clone()), *addr)).map_err(|e|e.to_string())?;
                                }
                            }
                            done = true;
                        }
                        if done 
                        {
                            pending_user_info_requests.remove_entry(&(info.name().to_string(),from));
                        }
                    }
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

                        let mut timed_out_pending_user_info_requests = Vec::new();
                        {
                            for (name, address) in &mut pending_user_info_requests.keys()
                            {
                                if let Some((from, time)) = pending_user_info_requests.get(&(name.clone(),*address))
                                {
                                    if time.elapsed() > Duration::from_millis(config.network.timeout_ms)
                                    {
                                        if let Some(addr) = from
                                        {
                                            sender_queue.send((Content::UserInfo(UserInfo::new_empty(name)), *addr)).map_err(|e|e.to_string())?;
                                        }
                                        timed_out_pending_user_info_requests.push((name.clone(),*address));
                                    }
                                }
                            }

                            for (name, address) in timed_out_pending_user_info_requests
                            {
                                pending_user_info_requests.remove(&(name.clone(),address));
                                log.log(MessageKind::Error, &format!("User info request for {} timed out", name))?;
                            }
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
                        request_connection(to, &mut pending_requests, &sender_queue, config.clone())?;
                    },
                    ConnectionRequest::Find(name) =>
                    {
                        find_user(None, &name, defines::MAX_FIND_TTL, &mut pending_user_info_requests, connection_list.clone(), &sender_queue)?;
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

fn find_user(
    from: Option<SocketAddr>,
    name: &str, 
    ttl: u8, 
    pending_user_info_requests: &mut HashMap::<(String,SocketAddr),(Option<SocketAddr>,Instant)>, 
    connection_list: Arc<RwLock<ConnectionList>>,
    sender_queue: &Sender<(Content,SocketAddr)>
) -> Result<(),String>
{
    let connection_list = connection_list.read().map_err(|e|e.to_string())?;
    if connection_list.get_address(name).is_some()
    {
        unreachable!("User is connected");
    }
    else
    {
        for c in connection_list.get_addresses()
        {
            if let Some((prev, _time)) = pending_user_info_requests.get(&(name.to_string(),c))
            {
                if prev.is_none()
                {
                    // we are already waiting for a response from this peer
                    return Ok(());
                }
            }
        }
        for c in connection_list.get_addresses()
        {
            pending_user_info_requests.insert((name.to_string(),c), (from,Instant::now()));
            sender_queue.send((Content::RequestUserInfo(name.to_string(), ttl), c)).map_err(|e|e.to_string())?;
        }
        Ok(()) 
    }

}

fn request_connection(
    to: SocketAddr,
    pending_requests: &mut HashMap::<SocketAddr,(Option<ContactInfo>,CryptoHandshakeInfo,Instant,u16)>,
    sender_queue: &Sender<(Content,SocketAddr)>,
    config: Arc<RwLock<Config>>
) -> Result<(),String>
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
    Ok(())
}