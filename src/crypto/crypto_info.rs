use crate::network::Serializable;

use super::{EcdhePublicKey, PublicKey};

#[derive(Clone, Debug)]
pub struct CryptoInfo
{
    nonce: u64,
    ecdhe_public_key: EcdhePublicKey,
    public_key: PublicKey
}

impl Serializable for CryptoInfo
{
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        todo!()
    }
}