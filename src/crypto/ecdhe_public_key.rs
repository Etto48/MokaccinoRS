use crate::network::Serializable;

#[derive(Clone, Debug)]
pub struct EcdhePublicKey
{
    ring_ecdh_public_key: ring::agreement::PublicKey
}

impl Serializable for EcdhePublicKey
{
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        todo!()
    }
}