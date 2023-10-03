use serializable::Serializable;

use crate::config::defines;

#[derive(Clone)]
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

impl Serializable for PublicKey
{
    fn serialize(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        ret.extend(self.key.serialize());
        ret
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        let (key, bytes_read) = Vec::<u8>::deserialize(data)?;
        Ok((Self { key }, bytes_read))
    }
}