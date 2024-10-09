use std::fmt;

use crate::word::Word;

#[derive(Debug)]
pub enum Error<const SIZE: usize> {
    UnmappedAddress(Word<SIZE>),
    InvalidAddress(u16),
    StackOverflow,
    StackUnderflow,
}

impl<const MEM_SIZE: usize> fmt::Display for Error<MEM_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnmappedAddress(address) => write!(f, "address 0x{address:04X} is not mapped to any region"),
            Error::InvalidAddress(address) => write!(f, "address 0x{address:04X} is out of memory bounds"),
            Error::StackOverflow => write!(f, "{self:?}"),
            Error::StackUnderflow => write!(f, "{self:?}"),
        }
    }
}

impl<const MEM_SIZE: usize> std::error::Error for Error<MEM_SIZE> {}

pub type Result<const MEM_SIZE: usize, T> = std::result::Result<T, Error<MEM_SIZE>>;
