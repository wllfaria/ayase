use crate::register::Register;
use crate::word::Word;

#[repr(u8)]
pub enum Instruction<const SIZE: usize> {
    MovLitReg(Register, u16),
    MovRegReg(Register, Register),
    PushLit(Word<SIZE>, u16),
}

impl<const SIZE: usize> std::fmt::Debug for Instruction<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::MovLitReg(reg, val) => write!(f, "mov {reg}, 0x{val:04X}"),
            Instruction::MovRegReg(from, to) => write!(f, "mov {from}, {to}"),
            Instruction::PushLit(address, val) => write!(f, "push 0x{val:04X} @ 0x{address:04X}"),
        }
    }
}
