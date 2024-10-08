use crate::register::Register;
use crate::word::Word;

#[repr(u8)]
pub enum Instruction<const SIZE: usize> {
    MovToReg(Register, Word<SIZE>),
}

impl<const SIZE: usize> std::fmt::Debug for Instruction<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::MovToReg(reg, val) => write!(f, "mov {reg}, 0x{val:04X}"),
        }
    }
}
