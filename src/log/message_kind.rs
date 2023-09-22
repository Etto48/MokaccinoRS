#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageKind
{
    Command,
    Event,
    Error,
}