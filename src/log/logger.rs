use std::{sync::{Arc, Mutex}, collections::LinkedList};

use super::{Log, MessageKind, LogMessage};

#[derive(Clone)]
pub struct Logger
{
    log: Arc<Mutex<Log>>,
    notification: Arc<Mutex<bool>>
}

impl Logger
{
    pub fn new() -> Self
    {
        Self { 
            log: Arc::new(Mutex::new(Log::new())),
            notification: Arc::new(Mutex::new(false))
        }
    }

    pub fn log(&self, message_kind: MessageKind, message: &str) -> Result<(),String>
    {
        let mut log = self.log.lock().map_err(|e|e.to_string())?;
        let mut notification = self.notification.lock().map_err(|e|e.to_string())?;
        log.add(LogMessage::new(message_kind,message));
        *notification = true;
        Ok(())
    }

    pub fn get(&self) -> Result<LinkedList<LogMessage>,String>
    {
        let log = self.log.lock().map_err(|e|e.to_string())?;
        let mut notification = self.notification.lock().map_err(|e|e.to_string())?;
        let messages = (*log.get()).clone();
        *notification = false;
        Ok(messages)
    }

    pub fn has_new_messages(&self) -> Result<bool,String>
    {
        let notification = self.notification.lock().map_err(|e|e.to_string())?;
        Ok(*notification)
    }
}