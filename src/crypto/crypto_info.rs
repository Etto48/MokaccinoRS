use serializable::Serializable;

use super::PublicKey;

#[derive(Serializable, Clone)]
pub struct CryptoInfo
{
    nonce: u64,
    ecdhe_public_key: PublicKey,
    public_key: PublicKey
}