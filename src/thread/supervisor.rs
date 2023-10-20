use std::{sync::{Arc, RwLock}, thread::JoinHandle};

use crate::{log::Logger, config::Config};

use super::threads::supervisor;

pub fn start(
    running: Arc<RwLock<bool>>,
    handles: Vec<JoinHandle<()>>,
    log: Logger,
    config: Arc<RwLock<Config>>
) -> JoinHandle<()>
{
    let builder = std::thread::Builder::new().name("Supervisor".to_string());
    match builder.spawn(move || {
        supervisor::run(
            running, 
            handles,
            log,
            config)
    })
    {
        Ok(handle) => handle,
        Err(e) => panic!("Error starting thread Supervisor: {e}")
    }   
}

#[cfg(test)]
mod tests {
    use crate::{thread::Context, log::MessageKind, config::defines};

    use super::*;

    #[test]
    fn supervisor() {
        let context = Context::new(None);
        let running1 = context.unmovable.running.clone();
        let running2 = context.unmovable.running.clone();

        let handles = vec![
            std::thread::spawn(move || {
                while *running1.read().unwrap() {
                    std::thread::sleep(defines::THREAD_QUEUE_TIMEOUT);
                }
            }),
            std::thread::spawn(|| {
                std::thread::sleep(defines::THREAD_QUEUE_TIMEOUT);
                panic!("Error")
            }),
            std::thread::spawn(move || {
                while *running2.read().unwrap() {
                    std::thread::sleep(defines::THREAD_QUEUE_TIMEOUT);
                }
            }),
        ];
        
        start(
            context.unmovable.running.clone(),
            handles,
            context.movable.log.clone(),
            context.unmovable.config.clone()
        );
        std::thread::sleep(defines::THREAD_QUEUE_TIMEOUT*5);
        context.unmovable.stop();
        std::thread::sleep(defines::THREAD_QUEUE_TIMEOUT*2);
        let log = context.movable.log.get().unwrap();
        let mut log_iter = log.iter();
        for l in log.iter() {
            println!("{:?}",l.text);
        }
        assert_eq!(log_iter.next().unwrap().kind, MessageKind::Event);
        assert_eq!(log_iter.next().unwrap().kind, MessageKind::Error);
    }
}