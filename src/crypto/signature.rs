use crate::network::Serializable;

pub struct Signature
{

}

impl Serializable for Signature
{
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        todo!()
    }
}