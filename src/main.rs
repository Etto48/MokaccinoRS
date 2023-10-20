#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::any::Any;

// hide console window on Windows in release
use mokaccino::{ui, thread, config::defines};


fn main() -> Result<(),Box<dyn Any + Send>>
{
    let mut threads: Vec<std::thread::JoinHandle<()>> = vec![];
    
    let context = thread::Context::new(Some(defines::CONFIG_PATH));

    threads.extend(thread::network::start(
        context.unmovable.running.clone(),
        context.movable.connection_list.clone(),
        context.movable.log.clone(),
        context.movable.text_queue_tx, 
        context.movable.connection_queue_tx, 
        context.movable.voice_queue_tx, 
        context.movable.sender_queue_rx, 
        context.unmovable.config.clone()
    ));

    threads.extend(thread::connection::start(
        context.unmovable.running.clone(),
        context.movable.connection_list.clone(),
        context.movable.log.clone(),
        context.movable.connection_requests_rx,
        context.movable.connection_queue_rx, 
        context.movable.sender_queue_tx.clone(), 
        context.unmovable.config.clone()
    ));

    threads.extend(thread::text::start(
        context.unmovable.running.clone(),
        context.movable.text_list.clone(),
        context.movable.connection_list.clone(),
        context.movable.log.clone(),
        context.movable.text_requests_rx,
        context.movable.text_queue_rx,
        context.movable.sender_queue_tx.clone(),
        context.unmovable.config.clone()
    ));

    threads.extend(thread::voice::start(
        context.unmovable.running.clone(),
        context.movable.connection_list.clone(),
        context.movable.log.clone(),
        context.movable.voice_requests_rx,
        context.movable.voice_interlocutor.clone(),
        context.movable.ui_notifications_tx.clone(),
        context.movable.voice_queue_rx, 
        context.movable.sender_queue_tx.clone(), 
        context.unmovable.config.clone()
    ));

    threads.extend(thread::file::start(
        context.unmovable.running.clone(),
        context.movable.connection_list.clone(),
        context.movable.log.clone(),
        context.movable.file_requests_rx,
        context.movable.file_queue_rx, 
        context.movable.sender_queue_tx, 
        context.unmovable.config.clone()
    ));

    let supervisor = thread::supervisor::start(
        context.unmovable.running.clone(),
        threads,
        context.movable.log.clone(),
        context.unmovable.config.clone()
    );

    // gui loop
    ui::run(
        context.movable.connection_list,
        context.movable.text_list,
        context.movable.log,
        context.movable.connection_requests_tx,
        context.movable.text_requests_tx,
        context.movable.voice_requests_tx,
        context.movable.voice_interlocutor,
        context.movable.ui_notifications_rx,
        
        context.unmovable,
    );

    match supervisor.join()
    {
        Ok(_) => 
        {
            println!("Thread Supervisor exited successfully");
            Ok(())
        },
        Err(e) => 
        {
            println!("Thread supervisor panicked: {}",e.downcast_ref::<String>().unwrap_or(&"Unknown".to_string()));
            Err(e)
        },
    }
}