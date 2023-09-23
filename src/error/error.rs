use std::fmt::Display;

use super::kind::ErrorKind;

#[derive(Debug, Clone)]
pub struct Error {
    message: String,
    kind: ErrorKind,
}

impl Display for Error
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for Error {}