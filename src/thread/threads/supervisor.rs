use std::{sync::{Arc, RwLock}, thread::JoinHandle};

use crate::{log::{Logger, MessageKind}, config::{Config, defines}};

pub fn run(
    running: Arc<RwLock<bool>>,
    handles: Vec<JoinHandle<Result<(),String>>>,
    log: Logger,
    _config: Arc<RwLock<Config>>
) -> Result<(),String>
{
    let mut thread_mask = vec![true;handles.len()];
    while *running.read().map_err(|e|e.to_string())?
    {
        for (i, handle) in handles.iter().enumerate()
        {
            if thread_mask[i]
            {
                let thread_id = format!("{:?}",handle.thread().id());
                let thread_name = handle.thread().name().unwrap_or(&thread_id);
                if handle.is_finished() && *running.read().map_err(|e|e.to_string())?
                {
                    log.log(MessageKind::Error,&format!("Thread {} exited unexpectedly",thread_name)).map_err(|e|e.to_string())?;
                    thread_mask[i] = false;
                }
            }
        }
        std::thread::sleep(defines::THREAD_SUPERVISOR_SLEEP_TIME);
    }
    'outer_loop: for handle in handles
    {
        let thread_id = format!("{:?}",handle.thread().id());
        let thread_name = handle.thread().name().unwrap_or(&thread_id).to_owned();
        let mut tries = 0;
        while !handle.is_finished()
        {
            if tries == 0
            {
                let message = format!("Waiting for thread {} to finish",thread_name);
                log.log(MessageKind::Event,&message).map_err(|e|e.to_string())?;
                println!("{}",message);
            }
            tries += 1;
            if tries > defines::MAX_THREAD_JOIN_TRIES
            {
                let message = format!("Thread {} did not finish in time",thread_name);
                log.log(MessageKind::Error,&message).map_err(|e|e.to_string())?;
                println!("{}",message);
                continue 'outer_loop;
            }
            std::thread::sleep(defines::THREAD_QUEUE_TIMEOUT);
        }
        match handle.join() 
        {
            Ok(ret) => 
            {
                match ret
                {
                    Ok(_) => 
                    {
                        let message = format!("Thread {} exited successfully",thread_name);
                        log.log(MessageKind::Event,&message).map_err(|e|e.to_string())?;
                        println!("{}",message);
                    },
                    Err(e) => 
                    {
                        let message = format!("Thread {} returned an error: {}",thread_name,e);
                        log.log(MessageKind::Error,&message).map_err(|e|e.to_string())?;
                        println!("{}",message);
                    }
                }
            },
            Err(e) => 
            {
                let message = format!("Thread {} panicked: {}",thread_name,e.downcast_ref::<String>().unwrap_or(&"Unknown".to_string()));
                log.log(MessageKind::Error,&message).map_err(|e|e.to_string())?;
                println!("{}",message);
            },
        }
    }
    Ok(())
}