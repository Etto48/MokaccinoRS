use std::collections::{HashMap, LinkedList};

use super::{TextInfo, TextFastStorage};

pub struct TextList
{
    messages: HashMap<String,TextFastStorage>,
    notifications: HashMap<String,bool>,
}

impl TextList
{
    pub fn new() -> Self
    {
        Self { 
            messages: HashMap::new(),
            notifications: HashMap::new(),
        }
    }

    pub fn add(&mut self, from: &str, text: TextInfo)
    {
        let storage = self.messages.entry(from.to_string()).or_insert(TextFastStorage::new());
        storage.add(text);
        self.notifications.insert(from.to_string(), true);
    }

    pub fn get(&mut self, from: &str) -> Option<&LinkedList<TextInfo>>
    {
        let ret = self.messages.get(from).map(|storage| storage.get());
        if ret.is_some()
        {
            self.notifications.insert(from.to_string(), false);
        }
        ret
    }

    pub fn contains(&self, from: &str, text: &TextInfo) -> bool
    {
        self.messages.get(from).map(|storage| storage.contains(text)).unwrap_or(false)
    }

    pub fn has_new_messages(&self, from: &str) -> bool
    {
        self.notifications.get(from).cloned().unwrap_or(false)
    }
}