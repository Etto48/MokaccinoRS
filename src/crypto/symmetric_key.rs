use std::error::Error;

use crate::config::defines;

use super::Ciphertext;

pub struct SymmetricKey
{
    key: Vec<u8>
}

impl SymmetricKey
{
    pub fn from_shared_secret(shared_secret: &[u8]) -> Self
    {
        let hss = openssl::hash::hash(openssl::hash::MessageDigest::sha3_512(), shared_secret).unwrap();
        SymmetricKey { key: hss[..defines::SYMMETRIC_ALGORITHM().key_len()].to_vec()}
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