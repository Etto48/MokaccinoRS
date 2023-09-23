use std::collections::{HashMap, LinkedList};

use super::{TextInfo, TextFastStorage};

pub struct TextList
{
    messages: HashMap<String,TextFastStorage>,
}

impl TextList
{
    pub fn new() -> Self
    {
        Self { 
            messages: HashMap::new(),
        }
    }

    pub fn add(&mut self, from: &str, text: TextInfo)
    {
        let storage = self.messages.entry(from.to_string()).or_insert(TextFastStorage::new());
        storage.add(text);
    }

    pub fn get(&self, from: &str) -> Option<&LinkedList<TextInfo>>
    {
        self.messages.get(from).map(|storage| storage.get())
    }

    pub fn contains(&self, from: &str, text: &TextInfo) -> bool
    {
        self.messages.get(from).map(|storage| storage.contains(text)).unwrap_or(false)
    }
}