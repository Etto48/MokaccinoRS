use crate::{config::Config, /*crypto::CryptoInfo*/};

use serializable::Serializable;


#[derive(Serializable, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContactInfo
{
    name: String,
    //crypto_info: CryptoInfo,
}


impl ContactInfo
{
    pub fn new(name: &str) -> Self
    {
        Self { name: name.to_string() }
    }

    pub fn from_config(config: &Config) -> Self
    {
        Self
        {
            name: config.network.name.to_string()
        }
    }

    pub fn name(&self) -> &str
    {
        &self.name
    }
}