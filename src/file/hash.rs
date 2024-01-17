use crate::config::defines;

pub fn hash(data: &[u8]) -> Vec<u8> {
    openssl::hash::hash(defines::MESSAGE_DIGEST(), data).unwrap().as_ref().to_vec()
}