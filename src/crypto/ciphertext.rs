use serializable::Serializable;

use crate::config::defines;

#[derive(Serializable, Clone)]
pub struct Ciphertext {
    pub ciphertext: Vec<u8>,
    pub iv: [u8; defines::SYMMETRIC_ALGORITHM_IV_LEN],
    pub tag: [u8; defines::SYMMETRIC_ALGORITHM_TAG_LEN],
}
