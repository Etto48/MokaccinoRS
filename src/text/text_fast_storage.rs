use std::collections::{HashSet, LinkedList};

use super::TextInfo;

pub struct TextFastStorage
{
    set: HashSet<TextInfo>,
    list: LinkedList<TextInfo>,
}

impl TextFastStorage
{
    pub fn new() -> Self
    {
        Self { 
            set: HashSet::new(),
            list: LinkedList::new(),
        }
    }

    pub fn add(&mut self, text: TextInfo)
    {
        if self.set.insert(text.clone())
        {
            self.list.push_back(text);
        }
    }

    pub fn get(&self) -> &LinkedList<TextInfo>
    {
        &self.list
    }

    pub fn contains(&self, text: &TextInfo) -> bool
    {
        self.set.contains(text)
    }
}

