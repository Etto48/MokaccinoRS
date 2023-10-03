use serializable::Serializable;

use crate::config::defines;

#[derive(Clone, Debug)]
pub struct EcdhePublicKey
{
    key: ring::agreement::UnparsedPublicKey<Vec<u8>>
}

impl Serializable for EcdhePublicKey
{
    fn serialize(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        ret.extend(self.key.bytes().serialize());
        ret
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        let (key, key_size) = Vec::<u8>::deserialize(data)?;
        let unparsed = ring::agreement::UnparsedPublicKey::new(defines::KEY_PAIR_ALGORITHM,key);
        Ok((Self { key: unparsed}, key_size))
    }
}