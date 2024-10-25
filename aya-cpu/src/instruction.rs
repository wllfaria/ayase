use crate::register::Register;
use crate::word::Word;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum InstructionSize {
    Small,
    Word,
}

#[derive(Debug)]
#[repr(u8)]
pub enum Instruction {
    MovLitReg(Register, u16),
    MovRegReg(Register, Register),
    MovRegMem(Register, Word),
    MovMemReg(Word, Register),
    MovLitMem(Word, u16),
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

    JeqLit(Word, u16),
    JeqReg(Word, Register),
    JgtLit(Word, u16),
    JgtReg(Word, Register),
    JneLit(Word, u16),
    JneReg(Word, Register),
    JgeLit(Word, u16),
    JgeReg(Word, Register),
    JleLit(Word, u16),
    JleReg(Word, Register),
    JltLit(Word, u16),
    JltReg(Word, Register),
    Jmp(Word),

    PushLit(u16),
    PopReg(Register),
    Call(Word),
    CallRegPtr(Register),
    Ret,
    Halt(u16),
}
