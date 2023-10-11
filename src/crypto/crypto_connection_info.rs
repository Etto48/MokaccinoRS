use serializable::Serializable;

use crate::config::Config;

use super::{PublicKey, CryptoLastingInfo};

#[derive(Serializable, Clone, Debug, PartialEq, Eq)]
pub struct CryptoConnectionInfo
{
    pub ecdhe_public_key: PublicKey,
    pub public_key: PublicKey,
}

impl CryptoConnectionInfo 
{
    pub fn from_config(config: &Config, ecdhe_public_key: PublicKey) -> Self
    {
        Self
        {
            ecdhe_public_key,
            public_key: config.network.private_key.public_key(),
        }
    }

    pub fn into_lasting(&self) -> CryptoLastingInfo
    {
        CryptoLastingInfo::new(&self.public_key)
    }
}