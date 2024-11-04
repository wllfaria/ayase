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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Register {
    Acc,
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
            Register::Acc => std::fmt::Display::fmt("ACC", f),
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
            Register::Acc,
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

impl From<Register> for u8 {
    fn from(register: Register) -> Self {
        register as u8
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
            0 => Ok(Register::Acc),
            2 => Ok(Register::R1),
            3 => Ok(Register::R2),
            4 => Ok(Register::R3),
            5 => Ok(Register::R4),
            6 => Ok(Register::R5),
            7 => Ok(Register::R6),
            8 => Ok(Register::R7),
            9 => Ok(Register::R8),
            1 => Ok(Register::IP),
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

impl TryFrom<u8> for Register {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Register::try_from(value as u16)
    }
}

impl TryFrom<&str> for Register {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "acc" => Ok(Self::Acc),
            "ip" => Ok(Self::IP),
            "r1" => Ok(Self::R1),
            "r2" => Ok(Self::R2),
            "r3" => Ok(Self::R3),
            "r4" => Ok(Self::R4),
            "r5" => Ok(Self::R5),
            "r6" => Ok(Self::R6),
            "r7" => Ok(Self::R7),
            "r8" => Ok(Self::R8),
            "sp" => Ok(Self::SP),
            "fp" => Ok(Self::FP),
            "ACC" => Ok(Self::Acc),
            "IP" => Ok(Self::IP),
            "R1" => Ok(Self::R1),
            "R2" => Ok(Self::R2),
            "R3" => Ok(Self::R3),
            "R4" => Ok(Self::R4),
            "R5" => Ok(Self::R5),
            "R6" => Ok(Self::R6),
            "R7" => Ok(Self::R7),
            "R8" => Ok(Self::R8),
            "SP" => Ok(Self::SP),
            "FP" => Ok(Self::FP),
            _ => Err(Error::InvalidRegister(format!(
                "value '{value}' is not a valid register name"
            ))),
        }
    }
}

#[derive(Debug)]
pub struct Registers {
    inner: [u16; Register::len()],
}

impl Registers {
    pub(crate) fn new(start_address: impl Into<Word>, stack_address: impl Into<Word>) -> Self {
        let mut registers = Self {
            inner: [0; Register::len()],
        };
        let stack_address = stack_address.into();
        registers.inner[Register::FP as usize] = u16::from(stack_address) - 2;
        registers.inner[Register::SP as usize] = u16::from(stack_address) - 2;
        registers.inner[Register::IP as usize] = start_address.into().into();
        registers
    }

    pub fn fetch_word(&self, register: Register) -> Word {
        assert!(matches!(register, Register::IP | Register::SP | Register::FP));
        let word = self.inner[register as usize];
        word.into()
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
