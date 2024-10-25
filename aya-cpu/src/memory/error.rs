use std::fmt;

use crate::word::Word;

#[derive(Debug)]
pub enum Error {
    UnmappedAddress(Word),
    InvalidAddress(u16),
    StackOverflow,
    StackUnderflow,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnmappedAddress(address) => write!(f, "address 0x{address:04X} is not mapped to any region"),
            Error::InvalidAddress(address) => write!(f, "address 0x{address:04X} is out of memory bounds"),
            Error::StackOverflow => write!(f, "{self:?}"),
            Error::StackUnderflow => write!(f, "{self:?}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
