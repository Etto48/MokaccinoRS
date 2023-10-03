use serializable::Serializable;

use super::{EcdhePublicKey, PublicKey};

#[derive(Serializable, Clone)]
pub struct CryptoInfo
{
    nonce: u64,
    ecdhe_public_key: EcdhePublicKey,
    public_key: PublicKey
}