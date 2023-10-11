use crate::{config::Config, crypto::{CryptoConnectionInfo, PublicKey}};
use serializable::Serializable;


#[derive(Serializable, Clone, Debug, PartialEq, Eq)]
pub struct ContactInfo
{
    name: String,
    crypto_info: CryptoConnectionInfo,
}


impl ContactInfo
{
    pub fn new(name: &str, info: &CryptoConnectionInfo) -> Self
    {
        Self { name: name.to_string() , crypto_info: info.clone() }
    }

    pub fn from_config(config: &Config, ecdhe_public_key: PublicKey) -> Self
    {
        Self
        {
            name: config.network.name.to_string(),
            crypto_info: CryptoConnectionInfo::from_config(config, ecdhe_public_key),
        }
    }

    pub fn name(&self) -> &str
    {
        &self.name
    }

    pub fn crypto_info(&self) -> &CryptoConnectionInfo
    {
        &self.crypto_info
    }
}