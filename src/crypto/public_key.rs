use serializable::Serializable;

use crate::config::defines;

#[derive(Serializable, Clone)]
pub struct PublicKey
{
    key: Vec<u8>
}

impl PublicKey 
{
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool
    {
        let key = ring::signature::UnparsedPublicKey::new(defines::SIGNING_ALGORITHM, &self.key);
        key.verify(data, signature).is_ok()
    }
}