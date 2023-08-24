#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use mokaccino::{ui, thread};


fn main() {
    let threads: Vec<std::thread::JoinHandle<Result<(),String>>> = vec![
        thread::connection::start(),
    ];

    ui::run();

    let mut success = true;
    for thread in threads {
        match thread.join() {
            Ok(Ok(())) => {},
            Ok(Err(e)) => {
                eprintln!("{}",e);
                success = false;
            },
            Err(e) => {
                eprintln!("Error joining thread: {:?}",e);
                success = false;
            }
        }
    }
    if !success {
        panic!("Error(s) occurred");
    }
}
