use crate::config::defines;

pub struct PrivateKey
{
    key: defines::PrivateKey
}

impl PrivateKey
{
    pub fn sign(&self, data: &[u8]) -> Vec<u8>
    {
        self.key.sign(data).as_ref().to_vec()
    }
}