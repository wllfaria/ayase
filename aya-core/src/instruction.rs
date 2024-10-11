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
    MovRegMem(Register, Word<SIZE>),
    MovMemReg(Word<SIZE>, Register),
    MovLitMem(Word<SIZE>, u16),
    MovRegPtrReg(Register, Register),

    AddRegReg(Register, Register),

    PushLit(u16),
    PopReg(Register),
    Call(Word<SIZE>),
    Ret,
    Halt(u16),
}

impl<const SIZE: usize> std::fmt::Debug for Instruction<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::MovLitReg(reg, val) => write!(f, "mov   {reg: <7} ${val:04X}"),
            Instruction::MovRegReg(from, to) => write!(f, "mov   {to: <7} {from}"),
            Instruction::MovRegMem(reg, address) => write!(f, "mov   &{address:04X} {reg}"),
            Instruction::MovMemReg(address, reg) => write!(f, "mov   {reg: <7} &{address:04X}"),
            Instruction::MovRegPtrReg(from, to) => write!(f, "mov   &{from: <7} {to}"),
            Instruction::MovLitMem(address, val) => write!(f, "mov   &{address:04X} ${val:04X}"),
            Instruction::AddRegReg(reg1, reg2) => write!(f, "add   {reg1} {reg2}"),

            Instruction::PushLit(val) => write!(f, "push  ${val:04X}"),
            Instruction::PopReg(reg) => write!(f, "pop      {reg}"),
            Instruction::Call(address) => write!(f, "call  &{address:04X}"),
            Instruction::Ret => write!(f, "ret"),
            Instruction::Halt(code) => write!(f, "hlt   ${code:04X}"),
        }
    }
}
