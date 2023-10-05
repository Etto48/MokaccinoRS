use crate::config::defines;

use super::{EcdhePublicKey, SymmetricKey};

#[derive(Debug)]
pub struct EcdhePrivateKey
{
}

impl EcdhePrivateKey
{
    pub fn derive(&self, public_key: EcdhePublicKey) -> std::io::Result<SymmetricKey>
    {
        todo!()
    }
}