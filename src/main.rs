#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// hide console window on Windows in release
use mokaccino::{ui, thread, config::defines};


fn main() {
    let mut threads: Vec<std::thread::JoinHandle<Result<(),String>>> = vec![];
    
    let context = thread::Context::new(Some("config.toml"));

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
        context.movable.voice_queue_rx, 
        context.movable.sender_queue_tx, 
        context.unmovable.config.clone()
    ));

    // gui loop
    ui::run(
        context.movable.connection_list,
        context.movable.text_list,
        context.movable.log,
        context.movable.connection_requests_tx,
        context.movable.text_requests_tx,
        
        context.unmovable,
    );

    // join all threads and check for errors
    let mut success = true;
    'thread_loop: for thread in threads {
        let thread_name = thread.thread().name().unwrap_or("unnamed").to_string();
        let mut join_tries = 0;
        while !thread.is_finished() {
            if join_tries > defines::MAX_THREAD_JOIN_TRIES
            {
                eprintln!("Thread {thread_name} timed out during join");
                success = false;
                continue 'thread_loop;
            }
            if join_tries == 0 
            {println!("Waiting for thread {thread_name} to finish");}
            std::thread::sleep(defines::THREAD_QUEUE_TIMEOUT);
            join_tries += 1;
        }
        match thread.join() {
            Ok(Ok(())) => {
                println!("Thread {thread_name} exited successfully");
            },
            Ok(Err(e)) => {
                eprintln!("Error occurred in thread {thread_name}: {e}");
                success = false;
            },
            Err(e) => {
                let default_error_string = "unknown cause".to_string();
                let error_string = e.downcast_ref::<String>().unwrap_or(&default_error_string);
                eprintln!("Error occurred in thread {thread_name}: {error_string}");
                success = false;
            }
        }
    }
    if !success {
        panic!("Error(s) occurred");
    }
}