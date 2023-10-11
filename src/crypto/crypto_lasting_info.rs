use serde::{Serialize, ser::SerializeSeq};
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
        let mut seq = serializer.serialize_seq(Some(1))?;
        seq.serialize_element(&self.public_key)?;
        seq.end()
    }
}

impl<'de> serde::Deserialize<'de> for CryptoLastingInfo
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>
    {
        struct CryptoLastingInfoVisitor;

        impl<'de> serde::de::Visitor<'de> for CryptoLastingInfoVisitor
        {
            type Value = CryptoLastingInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result
            {
                formatter.write_str("struct CryptoInfo")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de>
            {
                let public_key = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

                Ok(CryptoLastingInfo::new(&public_key))
            }
        }

        deserializer.deserialize_seq(CryptoLastingInfoVisitor)
    }
}

impl From<CryptoConnectionInfo> for CryptoLastingInfo
{
    fn from(info: CryptoConnectionInfo) -> Self
    {
        Self::new(&info.public_key)
    }
}