use std::{error::Error, str::FromStr};

use serde::{Serialize, Deserialize};
use serializable::Serializable;

use crate::config::defines;

#[derive(Clone, Debug)]
pub struct PublicKey
{
    pub(super) key: openssl::pkey::PKey<openssl::pkey::Public>,
}

impl PublicKey 
{

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool
    {
        match openssl::sign::Verifier::new(defines::MESSAGE_DIGEST(),&self.key)
        {
            Ok(mut verifier) => 
            {
                match verifier.update(data)
                {
                    Ok(_) => 
                    {
                        match verifier.verify(signature)
                        {
                            Ok(_) => 
                            {
                                true
                            },
                            Err(e) => 
                            {
                                println!("Verify Error: {}",e);
                                false
                            },
                        }
                    },
                    Err(e) => 
                    {
                        println!("Update Error: {}",e);
                        false
                    },
                }
            }
            Err(_) => false
        }
    }
}

impl Serializable for PublicKey
{
    fn serialize(&self) -> Vec<u8> {
        let der = self.key.public_key_to_der().unwrap();
        let mut ret = Vec::new();
        ret.extend(Serializable::serialize(&(der.len() as u32)));
        ret.extend(der);
        ret
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        let (len, offset) = <u32 as Serializable>::deserialize(data)?;
        if data.len() < offset + len as usize {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"Invalid data length"));
        }
        let key = openssl::pkey::PKey::public_key_from_der(&data[offset..offset+(len as usize)])
            .map_err(|err|{std::io::Error::new(std::io::ErrorKind::InvalidData,err)})?;
        Ok((PublicKey{key},offset+(len as usize)))
    }
}

impl ToString for PublicKey
{
    fn to_string(&self) -> String {
        openssl::base64::encode_block(&self.key.public_key_to_der().unwrap())
    }
}

impl FromStr for PublicKey
{
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let der = openssl::base64::decode_block(s)?;
        let key = openssl::pkey::PKey::public_key_from_der(&der)?;
        Ok(PublicKey{ key })
    }
}

impl PartialEq for PublicKey
{
    fn eq(&self, other: &Self) -> bool {
        self.key.public_key_to_der().unwrap() == other.key.public_key_to_der().unwrap()
    }
}

impl Eq for PublicKey {}

impl Serialize for PublicKey
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer, {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PublicKey
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let s = <String as Deserialize>::deserialize(deserializer)?;
        PublicKey::from_str(&s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}