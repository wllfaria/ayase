use crate::register::Register;
use crate::word::Word;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum InstructionSize {
    Small,
    Word,
}

#[derive(Debug)]
#[repr(u8)]
pub enum Instruction<const SIZE: usize> {
    MovLitReg(Register, u16),
    MovRegReg(Register, Register),
    MovRegMem(Register, Word<SIZE>),
    MovMemReg(Word<SIZE>, Register),
    MovLitMem(Word<SIZE>, u16),
    MovRegPtrReg(Register, Register),

    AddRegReg(Register, Register),
    AddLitReg(Register, u16),
    SubRegReg(Register, Register),
    SubLitReg(Register, u16),
    MulRegReg(Register, Register),
    MulLitReg(Register, u16),
    IncReg(Register),
    DecReg(Register),

    LshLitReg(Register, u16),
    LshRegReg(Register, Register),
    RshLitReg(Register, u16),
    RshRegReg(Register, Register),
    AndLitReg(Register, u16),
    AndRegReg(Register, Register),
    OrLitReg(Register, u16),
    OrRegReg(Register, Register),
    XorLitReg(Register, u16),
    XorRegReg(Register, Register),
    Not(Register),

    JeqLit(Word<SIZE>, u16),
    JeqReg(Word<SIZE>, Register),
    JgtLit(Word<SIZE>, u16),
    JgtReg(Word<SIZE>, Register),
    JneLit(Word<SIZE>, u16),
    JneReg(Word<SIZE>, Register),
    JgeLit(Word<SIZE>, u16),
    JgeReg(Word<SIZE>, Register),
    JleLit(Word<SIZE>, u16),
    JleReg(Word<SIZE>, Register),
    JltLit(Word<SIZE>, u16),
    JltReg(Word<SIZE>, Register),
    Jmp(Word<SIZE>),

    PushLit(u16),
    PopReg(Register),
    Call(Word<SIZE>),
    CallRegPtr(Register),
    Ret,
    Halt(u16),
}
