use std::error::Error;

use serializable::Serializable;

use crate::{config::defines, network::Packet};

use super::SymmetricKey;

#[derive(Serializable, Clone)]
pub struct Ciphertext {
    pub ciphertext: Vec<u8>,
    pub iv: [u8; defines::SYMMETRIC_ALGORITHM_IV_LEN],
    pub tag: [u8; defines::SYMMETRIC_ALGORITHM_TAG_LEN],
}

impl Ciphertext 
{
    pub fn from_packet(packet: Packet, key: &SymmetricKey) -> Self
    {
        let plaintext = packet.serialize();
        key.encrypt(&plaintext)
    }

    pub fn to_packet(self, key: &SymmetricKey) -> Result<Packet, Box<dyn Error>>
    {
        let plaintext = key.decrypt(&self)?;
        let (packet, len) = Packet::deserialize(&plaintext)?;
        if len != plaintext.len()
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"Packet length mismatch").into())
        }
        else 
        {
            Ok(packet)
        }
    }
}