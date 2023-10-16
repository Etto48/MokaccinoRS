use serializable::Serializable;

use crate::{config::Config, crypto::{SignedContactInfo, PublicKey}};

use super::ContactInfo;

#[derive(Serializable, Clone, Debug, PartialEq)]
pub enum Content
{
    Text(String,u64),
    AcknowledgeText(String,u64),
    Ping,
    Pong,
    RequestConnection(SignedContactInfo),
    AcknowledgeConnection,
    Voice(Vec<u8>),
    EndVoice,
}
impl Content {
    pub fn request_connection_from_config(config: &Config, ecdhe_public_key: PublicKey) -> Self {
        Content::RequestConnection(SignedContactInfo::from_contact_info(ContactInfo::from_config(config, ecdhe_public_key), &config.network.private_key))
    }
}