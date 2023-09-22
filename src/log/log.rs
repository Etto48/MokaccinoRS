use std::collections::LinkedList;

use super::{log_message::LogMessage, message_kind::MessageKind};

pub struct Log
{
    messages: LinkedList<LogMessage>,
}

impl Log
{
    pub fn new() -> Self
    {
        let mut ret = Self {
            messages: LinkedList::new(),
        };
        ret.add(LogMessage::new(MessageKind::Event,"Welcome to Mokaccino!"));
        ret
    }

    pub fn add(&mut self, message: LogMessage)
    {
        self.messages.push_back(message);
    }

    pub fn get(&self) -> &LinkedList<LogMessage>
    {
        &self.messages
    }
}
