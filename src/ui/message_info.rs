#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MessageDirection
{
    Incoming,
    Outgoing,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MessageInfo
{
    text: String,
    direction: MessageDirection
}

impl MessageInfo
{
    pub fn new(text: &str, direction: MessageDirection) -> Self
    {
        Self { text: text.to_string(), direction }
    }

    pub fn text(&self) -> &str
    {
        &self.text
    }

    pub fn direction(&self) -> MessageDirection
    {
        self.direction
    }
}