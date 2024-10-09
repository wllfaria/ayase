use std::fmt;

use crate::word::Word;

#[derive(Debug)]
pub enum Error {
    InvalidRegister(String),
    ForbiddenRegister(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum Register {
    Ret,
    IP,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    SP,
    FP,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register::Ret => std::fmt::Display::fmt("RET", f),
            Register::IP => std::fmt::Display::fmt("IP", f),
            Register::R1 => std::fmt::Display::fmt("R1", f),
            Register::R2 => std::fmt::Display::fmt("R2", f),
            Register::R3 => std::fmt::Display::fmt("R3", f),
            Register::R4 => std::fmt::Display::fmt("R4", f),
            Register::R5 => std::fmt::Display::fmt("R5", f),
            Register::R6 => std::fmt::Display::fmt("R6", f),
            Register::R7 => std::fmt::Display::fmt("R7", f),
            Register::R8 => std::fmt::Display::fmt("R8", f),
            Register::SP => std::fmt::Display::fmt("SP", f),
            Register::FP => std::fmt::Display::fmt("FP", f),
        }
    }
}

impl Register {
    pub const fn len() -> usize {
        12
    }

    pub const fn is_empty() -> bool {
        Register::len() == 0
    }

    pub fn iter() -> impl Iterator<Item = Register> {
        [
            Register::Ret,
            Register::IP,
            Register::R1,
            Register::R2,
            Register::R3,
            Register::R4,
            Register::R5,
            Register::R6,
            Register::R7,
            Register::R8,
            Register::SP,
            Register::FP,
        ]
        .into_iter()
    }
}

impl From<Register> for usize {
    fn from(register: Register) -> Self {
        register as usize
    }
}

impl From<Register> for u16 {
    fn from(val: Register) -> Self {
        val as u16
    }
}

impl TryFrom<u16> for Register {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            0 => Ok(Register::Ret),
            2 => Ok(Register::R1),
            3 => Ok(Register::R2),
            4 => Ok(Register::R3),
            5 => Ok(Register::R4),
            6 => Ok(Register::R5),
            7 => Ok(Register::R6),
            8 => Ok(Register::R7),
            9 => Ok(Register::R8),
            1 => Err(Error::ForbiddenRegister(format!(
                "access to register {} is forbidden",
                Register::IP
            ))),
            11 => Err(Error::ForbiddenRegister(format!(
                "access to register {} is forbidden",
                Register::SP
            ))),
            12 => Err(Error::ForbiddenRegister(format!(
                "access to register {} is forbidden",
                Register::FP
            ))),
            v => Err(Error::InvalidRegister(format!(
                "value 0x{v:04X} is not a valid register number"
            ))),
        }
    }
}

#[derive(Debug)]
pub struct Registers<const WORD_SIZE: usize> {
    inner: [u16; Register::len()],
}

impl<const WORD_SIZE: usize> Registers<WORD_SIZE> {
    pub(crate) fn new() -> Self {
        let mut registers = Self {
            inner: [0; Register::len()],
        };
        registers.inner[Register::FP as usize] = (WORD_SIZE - 2) as u16;
        registers.inner[Register::SP as usize] = (WORD_SIZE - 2) as u16;
        registers
    }

    pub fn fetch_word(&self, register: Register) -> Word<WORD_SIZE> {
        assert!(matches!(register, Register::IP | Register::SP | Register::FP));
        let word = self.inner[register as usize];
        assert!((word as usize) < WORD_SIZE);
        word.try_into().unwrap()
    }

    pub fn fetch(&self, register: Register) -> u16 {
        self.inner[register as usize]
    }

    pub(crate) fn set(&mut self, register: Register, value: u16) {
        self.inner[register as usize] = value;
    }

    #[cfg(debug_assertions)]
    pub fn inspect(&self) {
        for register in Register::iter() {
            println!("{: <3} @ 0x{:04X}", register, self.fetch(register));
        }
    }
}

impl<const WORD_SIZE: usize> Default for Registers<WORD_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}
