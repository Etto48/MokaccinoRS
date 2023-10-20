#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{any::Any, sync::{Arc, Mutex}};

// hide console window on Windows in release
use mokaccino::{ui, thread, config::defines};


fn main() -> Result<(),Box<dyn Any + Send>>
{
    let is_still_loading = Arc::new(Mutex::new(true));
    let is_still_loading_clone = is_still_loading.clone();
    let context = thread::Context::new(Some(defines::CONFIG_PATH));       

    let context_movable_connection_list_clone = context.movable.connection_list.clone();
    let context_movable_text_list_clone = context.movable.text_list.clone();
    let context_movable_log_clone = context.movable.log.clone();
    let context_movable_voice_interlocutor_clone = context.movable.voice_interlocutor.clone();
    let context_umovable_clone = context.unmovable.clone();

    let load_backend = std::thread::Builder::new().name("Loader".to_string()).spawn(move ||{
        let begin_time = std::time::Instant::now();

        let mut threads: Vec<std::thread::JoinHandle<()>> = vec![];
        
        

        threads.extend(thread::network::start(
            context_umovable_clone.running.clone(),
            context_movable_connection_list_clone.clone(),
            context_movable_log_clone.clone(),
            context.movable.text_queue_tx, 
            context.movable.file_queue_tx,
            context.movable.connection_queue_tx, 
            context.movable.voice_queue_tx, 
            context.movable.sender_queue_rx, 
            context_umovable_clone.config.clone()
        ));

        threads.extend(thread::connection::start(
            context_umovable_clone.running.clone(),
            context_movable_connection_list_clone.clone(),
            context_movable_log_clone.clone(),
            context.movable.connection_requests_rx,
            context.movable.connection_queue_rx, 
            context.movable.sender_queue_tx.clone(), 
            context_umovable_clone.config.clone()
        ));

        threads.extend(thread::text::start(
            context_umovable_clone.running.clone(),
            context_movable_text_list_clone,
            context_movable_connection_list_clone.clone(),
            context_movable_log_clone.clone(),
            context.movable.text_requests_rx,
            context.movable.text_queue_rx,
            context.movable.sender_queue_tx.clone(),
            context_umovable_clone.config.clone()
        ));

        threads.extend(thread::voice::start(
            context_umovable_clone.running.clone(),
            context_movable_connection_list_clone.clone(),
            context_movable_log_clone.clone(),
            context.movable.voice_requests_rx,
            context_movable_voice_interlocutor_clone,
            context.movable.ui_notifications_tx.clone(),
            context.movable.voice_queue_rx, 
            context.movable.sender_queue_tx.clone(), 
            context_umovable_clone.config.clone()
        ));

        threads.extend(thread::file::start(
            context_umovable_clone.running.clone(),
            context_movable_connection_list_clone,
            context_movable_log_clone.clone(),
            context.movable.file_requests_rx,
            context.movable.file_queue_rx, 
            context.movable.sender_queue_tx, 
            context_umovable_clone.config.clone()
        ));

        let supervisor = thread::supervisor::start(
            context_umovable_clone.running.clone(),
            threads,
            context_movable_log_clone,
            context_umovable_clone.config.clone()
        );
        if let Some(diff) = defines::MIN_LOAD_TIME.checked_sub(begin_time.elapsed())
        {
            std::thread::sleep(diff);
        }
        *is_still_loading_clone.lock().unwrap() = false;
        supervisor
    }).unwrap();

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
        is_still_loading,
    );

    let supervisor = load_backend.join().unwrap();

    // gui loop
    

    match supervisor.join()
    {
        Ok(_) => 
        {
            println!("Thread Supervisor exited successfully");
            Ok(())
        },
        Err(e) => 
        {
            let error_message = if let Some(&s) = e.downcast_ref::<&str>()
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
            println!("Thread supervisor panicked: {}", error_message);
            Err(e)
        },
    }
}