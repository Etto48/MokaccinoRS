use super::SymmetricKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CryptoSessionInfo
{
    pub symmetric_key: SymmetricKey,
}