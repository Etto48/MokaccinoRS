use std::time::SystemTime;

use super::MessageKind;

#[derive(Clone)]
pub struct LogMessage
{
    pub text: String,
    pub src: String,
    pub time: SystemTime,
    pub kind: MessageKind,
}

impl LogMessage
{
    pub fn new(message_kind: MessageKind, message: &str) -> Self
    {
        let thread_name = std::thread::current().name().unwrap_or("Unknown").to_string();
        Self {
            text: message.to_string(),
            src: thread_name,
            time: SystemTime::now(),
            kind: message_kind,
        }
    }
}