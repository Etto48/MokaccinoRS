use crate::config::config::Config;

use super::{Serializable, ContactInfo};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Content
{
    Text(String,u64),
    AcknowledgeText(String,u64),
    Ping,
    Pong,
    RequestConnection(ContactInfo),
    AcknowledgeConnection,
    VoiceRequest,
    VoiceAccept,
    VoiceAcknowledge,
    Voice(Vec<u8>),
}
impl Content {
    pub fn request_connection_from_config(config: &Config) -> Self {
        Content::RequestConnection(ContactInfo::from_config(config))
    }
}

impl Serializable for Content
{
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self
        {
            Content::Text(text,nonce) => {
                bytes.push(0);
                bytes.extend(text.serialize());
                bytes.extend(nonce.serialize());
            },
            Content::AcknowledgeText(text,nonce) => {
                bytes.push(1);
                bytes.extend(text.serialize());
                bytes.extend(nonce.serialize());
            },
            Content::Ping => {
                bytes.push(2);
            },
            Content::Pong => {
                bytes.push(3);
            },
            Content::RequestConnection(contact_info) => {
                bytes.push(4);
                bytes.extend(contact_info.serialize());
            },
            Content::AcknowledgeConnection => {
                bytes.push(5);
            },
            Content::VoiceRequest => {
                bytes.push(6);
            },
            Content::VoiceAccept => {
                bytes.push(7);
            },
            Content::VoiceAcknowledge => {
                bytes.push(8);
            },
            Content::Voice(voice) => {
                bytes.push(9);
                bytes.extend(voice.serialize());
            },
        }
        bytes
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        if data.len() < 1
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
        }
        else
        {
            match data[0]
            {
                0 => {
                    let (text, text_len) = String::deserialize(&data[1..])?;
                    let (nonce, nonce_len) = u64::deserialize(&data[text_len+1..])?;
                    Ok((Content::Text(text, nonce), text_len + nonce_len + 1))
                }
                1 => {
                    let (text, text_len) = String::deserialize(&data[1..])?;
                    let (nonce, nonce_len) = u64::deserialize(&data[text_len+1..])?;
                    Ok((Content::AcknowledgeText(text, nonce), text_len + nonce_len + 1))
                }
                2 => Ok((Content::Ping, 1)),
                3 => Ok((Content::Pong, 1)),
                4 => {
                    let (contact_info, contact_info_len) = ContactInfo::deserialize(&data[1..])?;
                    Ok((Content::RequestConnection(contact_info), contact_info_len + 1))
                }
                5 => Ok((Content::AcknowledgeConnection, 1)),
                6 => Ok((Content::VoiceRequest, 1)),
                7 => Ok((Content::VoiceAccept, 1)),
                8 => Ok((Content::VoiceAcknowledge, 1)),
                9 => {
                    let (voice, voice_len) = Vec::<u8>::deserialize(&data[1..])?;
                    Ok((Content::Voice(voice), voice_len + 1))
                }
                _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid content type"))
            }
        }
    }
}