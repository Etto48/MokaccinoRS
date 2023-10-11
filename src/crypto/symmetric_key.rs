use std::error::Error;

use crate::config::defines;

use super::Ciphertext;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SymmetricKey
{
    key: [u8; defines::SYMMETRIC_ALGORITHM_KEY_LEN]
}

impl SymmetricKey
{
    pub fn random() -> Self
    {
        let mut key = [0; defines::SYMMETRIC_ALGORITHM_KEY_LEN];
        openssl::rand::rand_bytes(&mut key).unwrap();
        SymmetricKey { key }
    }

    pub fn from_shared_secret(shared_secret: &[u8]) -> Self
    {
        let hss = openssl::hash::hash(openssl::hash::MessageDigest::sha3_512(), shared_secret).unwrap();
        SymmetricKey { key: hss[0..defines::SYMMETRIC_ALGORITHM_KEY_LEN].try_into().unwrap()}
    }

    pub fn encrypt(&self, data: &[u8]) -> Ciphertext
    {
        let mut iv = [0; defines::SYMMETRIC_ALGORITHM_IV_LEN];
        let mut tag = [0;defines::SYMMETRIC_ALGORITHM_TAG_LEN];
        openssl::rand::rand_bytes(&mut iv).unwrap();
        let ciphertext = openssl::symm::encrypt_aead(defines::SYMMETRIC_ALGORITHM(), &self.key, Some(&iv), &[], data, &mut tag).unwrap();
        Ciphertext {
            ciphertext,
            iv,
            tag,
        }
    }

    pub fn decrypt(&self, ciphertext: &Ciphertext) -> Result<Vec<u8>,Box<dyn Error>>
    {
        let plaintext = openssl::symm::decrypt_aead(defines::SYMMETRIC_ALGORITHM(), &self.key, Some(&ciphertext.iv), &[], &ciphertext.ciphertext, &ciphertext.tag)?;
        Ok(plaintext)
    }
}