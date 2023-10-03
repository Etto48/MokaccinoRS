use serializable::Serializable;

use crate::config::Config;

use super::ContactInfo;

#[derive(Serializable, Clone, Debug, PartialEq)]
pub enum Content
{
    Text(String,u64),
    AcknowledgeText(String,u64),
    Ping,
    Pong,
    RequestConnection(ContactInfo),
    AcknowledgeConnection,
    Voice(Vec<u8>),
}
impl Content {
    pub fn request_connection_from_config(config: &Config) -> Self {
        Content::RequestConnection(ContactInfo::from_config(config))
    }
}