use std::error::Error;

use super::{PrivateKey, PublicKey, SymmetricKey};

pub struct CryptoHandshakeInfo
{
    pub local_ecdhe_key: PrivateKey,
    pub remote_ecdhe_key: Option<PublicKey>,
}

impl CryptoHandshakeInfo
{
    pub fn derive(&self) -> Result<SymmetricKey,Box<dyn Error>>
    {
        if let Some(remote_ecdhe_key) = &self.remote_ecdhe_key
        {
            Ok(self.local_ecdhe_key.derive(remote_ecdhe_key.clone())?)
        }
        else
        {
            Err("Remote ECDHE key not set".into())
        }
    }
}