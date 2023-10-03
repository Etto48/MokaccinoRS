use serializable::Serializable;

#[derive(Serializable, Clone, Debug)]
pub struct EcdhePublicKey
{
    key: Vec<u8>
}