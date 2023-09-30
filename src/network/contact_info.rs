use crate::{config::Config, /*crypto::CryptoInfo*/};

use super::Serializable;


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

impl Serializable for ContactInfo
{
    fn serialize(&self) -> Vec<u8> {
        self.name.serialize()
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        let (name, bytes_read) = String::deserialize(data)?;
        Ok((Self::new(&name), bytes_read))
    }
}