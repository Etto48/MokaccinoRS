use std::{error::Error, str::FromStr};

use serializable::Serializable;

#[derive(Clone)]
pub struct PublicKey
{
    pub key: openssl::pkey::PKey<openssl::pkey::Public>,
}

impl PublicKey 
{
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool
    {
        if let Ok(mut verifier) = openssl::sign::Verifier::new(openssl::hash::MessageDigest::sha3_512(), &self.key)
        {
            verifier.verify_oneshot(signature, data).unwrap_or(false)
        }
        else
        {
            false
        }
    }
}

impl Serializable for PublicKey
{
    fn serialize(&self) -> Vec<u8> {
        let der = self.key.public_key_to_der().unwrap();
        let mut ret = Vec::new();
        ret.extend((der.len() as u32).serialize());
        ret.extend(der);
        ret
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        let (len, offset) = u32::deserialize(data)?;
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