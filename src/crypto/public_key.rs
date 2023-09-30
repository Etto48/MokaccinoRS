use crate::network::Serializable;

use super::Signature;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PublicKey
{
    
}

impl PublicKey 
{
    pub fn verify(&self, data: &[u8], signature: &Signature) -> bool
    {
        todo!()
    }
}

impl Serializable for PublicKey
{
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        todo!()
    }
}