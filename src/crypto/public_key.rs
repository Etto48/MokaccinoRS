use serializable::Serializable;

use crate::config::defines;

#[derive(Serializable, Clone)]
pub struct PublicKey
{
}

impl PublicKey 
{
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool
    {
        todo!()
    }
}