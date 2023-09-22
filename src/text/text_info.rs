#[derive(Clone, Eq, PartialEq, Hash)]
pub enum TextDirection
{
    Incoming,
    Outgoing,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TextInfo
{
    pub text: String,
    pub nonce: u64,
    pub direction: TextDirection,
}