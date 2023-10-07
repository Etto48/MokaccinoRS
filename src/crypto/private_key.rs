use std::{error::Error, str::FromStr};

use super::{PublicKey, SymmetricKey};

pub struct PrivateKey
{
    key: openssl::pkey::PKey<openssl::pkey::Private>,
}

impl PrivateKey
{
    pub fn new() -> Self
    {
        PrivateKey{ key: openssl::pkey::PKey::generate_x448().unwrap() }
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8>
    {
        let mut signer = openssl::sign::Signer::new(openssl::hash::MessageDigest::sha3_512(), &self.key).unwrap();
        let mut signature = vec![0;signer.len().unwrap()];
        signer.sign_oneshot(&mut signature, data).unwrap();
        signature
    }

    pub fn public_key(&self) -> PublicKey
    {
        let der = self.key.public_key_to_der().unwrap();
        let key = openssl::pkey::PKey::public_key_from_der(&der).unwrap();
        PublicKey { key }
    }

    pub fn derive(&self, public_key: PublicKey) -> Result<SymmetricKey,Box<dyn Error>>
    {
        let mut deriver = openssl::derive::Deriver::new(&self.key).unwrap();
        deriver.set_peer(&public_key.key)?;
        let shared_secret = deriver.derive_to_vec()?;
        Ok(SymmetricKey::from_shared_secret(&shared_secret))
    }
}

impl ToString for PrivateKey
{
    fn to_string(&self) -> String {
        openssl::base64::encode_block(&self.key.private_key_to_der().unwrap())
    }
}

impl FromStr for PrivateKey
{
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let der = openssl::base64::decode_block(s)?;
        let key = openssl::pkey::PKey::private_key_from_der(&der)?;
        Ok(PrivateKey{ key })
    }
}