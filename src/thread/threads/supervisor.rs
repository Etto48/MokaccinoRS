use std::{sync::{Arc, RwLock}, thread::JoinHandle};

use crate::{log::{Logger, MessageKind}, config::{Config, defines}};

pub fn run(
    running: Arc<RwLock<bool>>,
    mut handles: Vec<JoinHandle<()>>,
    log: Logger,
    _config: Arc<RwLock<Config>>
)
{
    while *running.read().unwrap()
    {
        let mut to_join = Vec::new();
        for (i, handle) in handles.iter().enumerate()
        {
            if handle.is_finished() && *running.read().unwrap()
            {
                to_join.push(i);
            }
        }
        for i in to_join
        {
            let handle = handles.remove(i);
            join_and_display(handle, &log).unwrap();
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
                log.log(MessageKind::Event,&message).unwrap();
                println!("{}",message);
            }
            tries += 1;
            if tries > defines::MAX_THREAD_JOIN_TRIES
            {
                let message = format!("Thread {} did not finish in time",thread_name);
                log.log(MessageKind::Error,&message).unwrap();
                println!("{}",message);
                continue 'outer_loop;
            }
            std::thread::sleep(defines::THREAD_QUEUE_TIMEOUT);
        }
        join_and_display(handle, &log).unwrap();
    }
}

fn join_and_display(handle: JoinHandle<()>, log: &Logger) -> Result<(),String>
{
    let thread_id = format!("{:?}",handle.thread().id());
    let thread_name = handle.thread().name().unwrap_or(&thread_id).to_owned();
    match handle.join() 
    {
        Ok(_) => 
        {
            let message = format!("Thread {} exited successfully",thread_name);
            log.log(MessageKind::Event,&message).unwrap();
            println!("{}",message);
        },
        Err(e) => 
        {
            //let error_string = e.downcast::<String>().unwrap_or("Unknown".to_string().into());
            let error_message =
            if let Some(&s) = e.downcast_ref::<&str>()
            {
                s
            }
            else if let Some(s) = e.downcast_ref::<String>()
            {
                s.as_str()
            }
            else
            {
                "Unknown"
            };

            let message = format!("Thread {} panicked: {:?}",thread_name,error_message);
            log.log(MessageKind::Error,&message).unwrap();
            println!("{}",message);
        },
    }
    Ok(())
}