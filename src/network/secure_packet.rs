use serializable::Serializable;

use crate::crypto::Ciphertext;

use super::Packet;

#[derive(Serializable, Clone)]
pub enum SecurePacket
{
    Ciphertext(Ciphertext),
    Plaintext(Packet),
}