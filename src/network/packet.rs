use std::time::SystemTime;

use serializable::Serializable;

use super::Content;


#[derive(Serializable, Clone, Debug, PartialEq)]
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