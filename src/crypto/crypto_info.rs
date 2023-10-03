use serializable::Serializable;

use super::{EcdhePublicKey, PublicKey};

#[derive(Clone)]
pub struct CryptoInfo
{
    nonce: u64,
    ecdhe_public_key: EcdhePublicKey,
    public_key: PublicKey
}

impl Serializable for CryptoInfo
{
    fn serialize(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        ret.extend(self.nonce.serialize());
        ret.extend(self.ecdhe_public_key.serialize());
        ret.extend(self.public_key.serialize());
        ret
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        let (nonce, nonce_size) = u64::deserialize(data)?;
        let (ecdhe_public_key, ecdhe_public_key_size) = EcdhePublicKey::deserialize(&data[nonce_size..])?;
        let (public_key, public_key_size) = PublicKey::deserialize(&data[nonce_size + ecdhe_public_key_size..])?;
        Ok((Self { nonce, ecdhe_public_key, public_key }, nonce_size + ecdhe_public_key_size + public_key_size))
    }
}