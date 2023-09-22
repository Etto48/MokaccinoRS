use std::{sync::{Arc, Mutex}, collections::LinkedList};

use super::{log::Log, message_kind::MessageKind, log_message::LogMessage};

#[derive(Clone)]
pub struct Logger
{
    log: Arc<Mutex<Log>>,
}

impl Logger
{
    pub fn new() -> Self
    {
        Self { 
            log: Arc::new(Mutex::new(Log::new())),
        }
    }

    pub fn log(&self, message_kind: MessageKind, message: &str) -> Result<(),String>
    {
        let mut log = self.log.lock().map_err(|e|e.to_string())?;
        log.add(LogMessage::new(message_kind,message));
        Ok(())
    }

    pub fn get(&self) -> Result<LinkedList<LogMessage>,String>
    {
        let log = self.log.lock().map_err(|e|e.to_string())?;
        let messages = (*log.get()).clone();
        Ok(messages)
    }
}