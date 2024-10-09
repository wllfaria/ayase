use std::fmt;

#[derive(Debug)]
pub enum Error {
    UnmappedAddress(String),
    InvalidAddress(String),
    StackOverflow,
    StackUnderflow,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
