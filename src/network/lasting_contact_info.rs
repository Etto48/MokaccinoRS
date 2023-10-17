use serde::{Serialize, Deserialize};
use serializable::Serializable;

use crate::crypto::CryptoLastingInfo;

#[derive(Serializable, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastingContactInfo
{
    name: String,
    crypto_info: CryptoLastingInfo,
}

impl LastingContactInfo
{
    pub fn new(name: &str, info: &CryptoLastingInfo) -> Self
    {
        Self { name: name.to_string() , crypto_info: info.clone() }
    }

    pub fn name(&self) -> &str
    {
        &self.name
    }

    pub fn crypto_info(&self) -> &CryptoLastingInfo
    {
        &self.crypto_info
    }
}