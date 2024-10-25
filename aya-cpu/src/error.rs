use std::fmt;

use crate::{memory, op_code, register};

#[derive(Debug)]
pub enum Error {
    Mem(memory::Error),
    OpCode(op_code::Error),
    Register(register::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl From<memory::Error> for Error {
    fn from(err: memory::Error) -> Self {
        Self::Mem(err)
    }
}

impl From<op_code::Error> for Error {
    fn from(err: op_code::Error) -> Self {
        Self::OpCode(err)
    }
}

impl From<register::Error> for Error {
    fn from(err: register::Error) -> Self {
        Self::Register(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
