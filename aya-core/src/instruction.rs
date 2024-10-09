use crate::register::Register;
use crate::word::Word;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum InstructionSize {
    Small,
    Word,
}

#[repr(u8)]
pub enum Instruction<const SIZE: usize> {
    MovLitReg(Register, u16),
    MovRegReg(Register, Register),
    PushLit(u16),
    Pop,
    PopReg(Register),
    Call(Word<SIZE>),
    Ret,
    Halt(u16),
}

impl<const SIZE: usize> std::fmt::Debug for Instruction<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::MovLitReg(reg, val) => write!(f, "mov   {reg: <7} 0x{val:04X}"),
            Instruction::MovRegReg(from, to) => write!(f, "mov   {from: <7} {to}"),
            Instruction::PushLit(val) => write!(f, "push  0x{val:04X}"),
            Instruction::Pop => write!(f, "pop"),
            Instruction::PopReg(reg) => write!(f, "pop      {reg}"),
            Instruction::Call(address) => write!(f, "call  0x{address:04X}"),
            Instruction::Ret => write!(f, "ret"),
            Instruction::Halt(code) => write!(f, "hlt   0x{code:04X}"),
        }
    }
}
