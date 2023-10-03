use std::time::SystemTime;

use serializable::Serializable;

use super::Content;


#[derive(Clone, Debug, PartialEq)]
pub struct Packet
{
    pub content: Content,
    pub timestamp: SystemTime,
}

impl Packet
{
    pub fn from_content_now(content: Content) -> Self
    {
        Self
        {
            content, 
            timestamp: SystemTime::now()
        }
    }
}

impl Serializable for Packet
{
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.content.serialize());
        bytes.extend(self.timestamp.serialize());
        bytes
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        let (content, content_len) = Content::deserialize(data)?;
        let (timestamp, timestamp_len) = SystemTime::deserialize(&data[content_len..])?;
        Ok((Packet { content, timestamp }, content_len + timestamp_len))
    }
}