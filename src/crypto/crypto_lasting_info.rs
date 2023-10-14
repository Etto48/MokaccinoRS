use serde::{Serialize, Deserialize};
use serializable::Serializable;

use crate::config;

use super::{PublicKey, CryptoConnectionInfo};

#[derive(Serializable, Clone, Debug, PartialEq, Eq)]
pub struct CryptoLastingInfo
{
    pub public_key: PublicKey
}

impl CryptoLastingInfo
{
    pub fn new(public_key: &PublicKey) -> Self
    {
        Self
        {
            public_key: public_key.clone(),
        }
    }

    pub fn from_config(config: &config::Config) -> Self
    {
        Self::new(
            &config.network.private_key.public_key().into(),
        )
    }
}

impl Serialize for CryptoLastingInfo
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer
    {
        Serialize::serialize(&self.public_key, serializer)
    }
}

impl<'de> Deserialize<'de> for CryptoLastingInfo
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>
    {
        let public_key = <PublicKey as Deserialize<'de>>::deserialize(deserializer)?;
        Ok(Self::new(&public_key))
    }
}

impl From<CryptoConnectionInfo> for CryptoLastingInfo
{
    fn from(info: CryptoConnectionInfo) -> Self
    {
        Self::new(&info.public_key)
    }
}