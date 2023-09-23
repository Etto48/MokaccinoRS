use super::TextDirection;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TextInfo
{
    pub text: String,
    pub nonce: u64,
    pub direction: TextDirection,
}