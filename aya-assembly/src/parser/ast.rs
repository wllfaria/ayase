use std::ops::Range;

use aya_cpu::op_code::OpCode;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mul,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ByteOffset {
    pub start: usize,
    pub end: usize,
}

impl From<ByteOffset> for Range<usize> {
    fn from(offset: ByteOffset) -> Range<usize> {
        offset.start..offset.end
    }
}

impl From<ByteOffset> for miette::SourceSpan {
    fn from(value: ByteOffset) -> Self {
        Self::new(value.start.into(), value.end - value.start)
    }
}

impl From<Range<usize>> for ByteOffset {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

#[derive(Debug)]
pub struct Ast {
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Instruction(Box<Instruction>),
    HexLiteral(ByteOffset),
    Address(ByteOffset),
    Register(ByteOffset),
    Var(ByteOffset),
    Operator(Operator),
    Label {
        name: ByteOffset,
        exported: bool,
    },
    FieldAccessor {
        module: ByteOffset,
        field: ByteOffset,
    },
    Import {
        name: ByteOffset,
        path: ByteOffset,
        address: ByteOffset,
        variables: Vec<Statement>,
    },
    ImportVar {
        name: ByteOffset,
        value: Box<Statement>,
    },
    Data {
        name: ByteOffset,
        size: u8,
        exported: bool,
        values: Vec<Statement>,
    },
    Const {
        name: ByteOffset,
        exported: bool,
        value: Box<Statement>,
    },
    BinaryOp {
        lhs: Box<Statement>,
        operator: Operator,
        rhs: Box<Statement>,
    },
}

impl From<Instruction> for Statement {
    fn from(instruction: Instruction) -> Self {
        Self::Instruction(Box::new(instruction))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum InstructionKind {
    LitReg,
    RegReg,
    RegMem,
    MemReg,
    LitMem,
    RegPtrReg,
    NoArgs,
    SingleReg,
    SingleLit,
}

impl InstructionKind {
    pub fn byte_size(&self) -> u8 {
        match self {
            InstructionKind::LitReg => 4,
            InstructionKind::RegReg => 3,
            InstructionKind::RegMem => 4,
            InstructionKind::MemReg => 4,
            InstructionKind::LitMem => 5,
            InstructionKind::RegPtrReg => 3,
            InstructionKind::NoArgs => 1,
            InstructionKind::SingleReg => 2,
            InstructionKind::SingleLit => 3,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    MovLitReg(Statement, Statement),
    MovRegReg(Statement, Statement),
    MovRegMem(Statement, Statement),
    MovMemReg(Statement, Statement),
    MovLitMem(Statement, Statement),
    MovRegPtrReg(Statement, Statement),
    AddRegReg(Statement, Statement),
    AddLitReg(Statement, Statement),
    SubRegReg(Statement, Statement),
    SubLitReg(Statement, Statement),
    MulRegReg(Statement, Statement),
    MulLitReg(Statement, Statement),
    LshRegReg(Statement, Statement),
    LshLitReg(Statement, Statement),
    RshRegReg(Statement, Statement),
    RshLitReg(Statement, Statement),
    AndRegReg(Statement, Statement),
    AndLitReg(Statement, Statement),
    OrLitReg(Statement, Statement),
    OrRegReg(Statement, Statement),
    XorLitReg(Statement, Statement),
    XorRegReg(Statement, Statement),
    Inc(Statement),
    Dec(Statement),
    Not(Statement),
    JeqLit(Statement, Statement),
    JeqReg(Statement, Statement),
    JgtLit(Statement, Statement),
    JgtReg(Statement, Statement),
    JneLit(Statement, Statement),
    JneReg(Statement, Statement),
    JgeLit(Statement, Statement),
    JgeReg(Statement, Statement),
    JleLit(Statement, Statement),
    JleReg(Statement, Statement),
    JltLit(Statement, Statement),
    JltReg(Statement, Statement),
    Jmp(Statement),
    PshLit(Statement),
    PshReg(Statement),
    Pop(Statement),
    CallLit(Statement),
    Ret,
    Hlt,
}

impl Instruction {
    pub fn lhs(&self) -> &Statement {
        match self {
            Instruction::MovLitReg(lhs, _)
            | Instruction::MovRegReg(lhs, _)
            | Instruction::MovRegMem(lhs, _)
            | Instruction::MovMemReg(lhs, _)
            | Instruction::MovLitMem(lhs, _)
            | Instruction::MovRegPtrReg(lhs, _)
            | Instruction::AddRegReg(lhs, _)
            | Instruction::AddLitReg(lhs, _)
            | Instruction::SubRegReg(lhs, _)
            | Instruction::SubLitReg(lhs, _)
            | Instruction::MulRegReg(lhs, _)
            | Instruction::MulLitReg(lhs, _)
            | Instruction::LshRegReg(lhs, _)
            | Instruction::LshLitReg(lhs, _)
            | Instruction::RshRegReg(lhs, _)
            | Instruction::RshLitReg(lhs, _)
            | Instruction::AndRegReg(lhs, _)
            | Instruction::AndLitReg(lhs, _)
            | Instruction::OrLitReg(lhs, _)
            | Instruction::OrRegReg(lhs, _)
            | Instruction::XorLitReg(lhs, _)
            | Instruction::XorRegReg(lhs, _)
            | Instruction::JeqLit(lhs, _)
            | Instruction::JeqReg(lhs, _)
            | Instruction::JgtLit(lhs, _)
            | Instruction::JgtReg(lhs, _)
            | Instruction::JneLit(lhs, _)
            | Instruction::JneReg(lhs, _)
            | Instruction::JgeLit(lhs, _)
            | Instruction::JgeReg(lhs, _)
            | Instruction::JleLit(lhs, _)
            | Instruction::JleReg(lhs, _)
            | Instruction::JltLit(lhs, _)
            | Instruction::JltReg(lhs, _)
            | Instruction::PshLit(lhs)
            | Instruction::PshReg(lhs)
            | Instruction::Pop(lhs)
            | Instruction::CallLit(lhs)
            | Instruction::Inc(lhs)
            | Instruction::Dec(lhs)
            | Instruction::Jmp(lhs)
            | Instruction::Not(lhs) => lhs,

            Instruction::Ret | Instruction::Hlt => unreachable!(),
        }
    }

    pub fn rhs(&self) -> &Statement {
        match self {
            Instruction::MovLitReg(_, rhs)
            | Instruction::MovRegReg(_, rhs)
            | Instruction::MovRegMem(_, rhs)
            | Instruction::MovMemReg(_, rhs)
            | Instruction::MovLitMem(_, rhs)
            | Instruction::MovRegPtrReg(_, rhs)
            | Instruction::AddRegReg(_, rhs)
            | Instruction::AddLitReg(_, rhs)
            | Instruction::SubRegReg(_, rhs)
            | Instruction::SubLitReg(_, rhs)
            | Instruction::MulRegReg(_, rhs)
            | Instruction::MulLitReg(_, rhs)
            | Instruction::LshRegReg(_, rhs)
            | Instruction::LshLitReg(_, rhs)
            | Instruction::RshRegReg(_, rhs)
            | Instruction::RshLitReg(_, rhs)
            | Instruction::AndRegReg(_, rhs)
            | Instruction::AndLitReg(_, rhs)
            | Instruction::OrLitReg(_, rhs)
            | Instruction::OrRegReg(_, rhs)
            | Instruction::XorLitReg(_, rhs)
            | Instruction::XorRegReg(_, rhs)
            | Instruction::JeqLit(_, rhs)
            | Instruction::JeqReg(_, rhs)
            | Instruction::JgtLit(_, rhs)
            | Instruction::JgtReg(_, rhs)
            | Instruction::JneLit(_, rhs)
            | Instruction::JneReg(_, rhs)
            | Instruction::JgeLit(_, rhs)
            | Instruction::JgeReg(_, rhs)
            | Instruction::JleLit(_, rhs)
            | Instruction::JleReg(_, rhs)
            | Instruction::JltLit(_, rhs)
            | Instruction::JltReg(_, rhs) => rhs,

            Instruction::PshLit(_)
            | Instruction::PshReg(_)
            | Instruction::Pop(_)
            | Instruction::CallLit(_)
            | Instruction::Inc(_)
            | Instruction::Dec(_)
            | Instruction::Not(_)
            | Instruction::Jmp(_)
            | Instruction::Ret
            | Instruction::Hlt => unreachable!(),
        }
    }

    pub fn opcode(&self) -> OpCode {
        match self {
            Instruction::MovLitReg(_, _) => OpCode::MovLitReg,
            Instruction::MovRegReg(_, _) => OpCode::MovRegReg,
            Instruction::MovRegMem(_, _) => OpCode::MovRegMem,
            Instruction::MovMemReg(_, _) => OpCode::MovMemReg,
            Instruction::MovLitMem(_, _) => OpCode::MovLitMem,
            Instruction::MovRegPtrReg(_, _) => OpCode::MovRegPtrReg,

            Instruction::AddRegReg(_, _) => OpCode::AddRegReg,
            Instruction::AddLitReg(_, _) => OpCode::AddLitReg,
            Instruction::SubRegReg(_, _) => OpCode::SubRegReg,
            Instruction::SubLitReg(_, _) => OpCode::SubLitReg,
            Instruction::Inc(_) => OpCode::IncReg,
            Instruction::Dec(_) => OpCode::DecReg,
            Instruction::MulLitReg(_, _) => OpCode::MulLitReg,
            Instruction::MulRegReg(_, _) => OpCode::MulRegReg,

            Instruction::LshLitReg(_, _) => OpCode::LshLitReg,
            Instruction::LshRegReg(_, _) => OpCode::LshRegReg,
            Instruction::RshLitReg(_, _) => OpCode::RshLitReg,
            Instruction::RshRegReg(_, _) => OpCode::RshRegReg,
            Instruction::AndLitReg(_, _) => OpCode::AndLitReg,
            Instruction::AndRegReg(_, _) => OpCode::AndRegReg,
            Instruction::OrLitReg(_, _) => OpCode::OrLitReg,
            Instruction::OrRegReg(_, _) => OpCode::OrRegReg,
            Instruction::XorLitReg(_, _) => OpCode::XorLitReg,
            Instruction::XorRegReg(_, _) => OpCode::XorRegReg,
            Instruction::Not(_) => OpCode::Not,

            Instruction::PshLit(_) => OpCode::PushLit,
            Instruction::PshReg(_) => OpCode::PushReg,
            Instruction::Pop(_) => OpCode::Pop,
            Instruction::CallLit(_) => OpCode::Call,
            Instruction::Ret => OpCode::Ret,
            Instruction::Hlt => OpCode::Halt,

            Instruction::JeqLit(_, _) => OpCode::JeqLit,
            Instruction::JeqReg(_, _) => OpCode::JeqReg,
            Instruction::JgtLit(_, _) => OpCode::JgtLit,
            Instruction::JgtReg(_, _) => OpCode::JgtReg,
            Instruction::JneLit(_, _) => OpCode::JneLit,
            Instruction::JneReg(_, _) => OpCode::JneReg,
            Instruction::JgeLit(_, _) => OpCode::JgeLit,
            Instruction::JgeReg(_, _) => OpCode::JgeReg,
            Instruction::JleLit(_, _) => OpCode::JleLit,
            Instruction::JleReg(_, _) => OpCode::JleReg,
            Instruction::JltLit(_, _) => OpCode::JltLit,
            Instruction::JltReg(_, _) => OpCode::JltReg,
            Instruction::Jmp(_) => OpCode::Jmp,
        }
    }

    pub fn kind(&self) -> InstructionKind {
        match self {
            Instruction::MovLitReg(_, _)
            | Instruction::AddLitReg(_, _)
            | Instruction::SubLitReg(_, _)
            | Instruction::MulLitReg(_, _)
            | Instruction::AndLitReg(_, _)
            | Instruction::OrLitReg(_, _)
            | Instruction::LshLitReg(_, _)
            | Instruction::RshLitReg(_, _)
            | Instruction::XorLitReg(_, _) => InstructionKind::LitReg,

            Instruction::MovRegReg(_, _)
            | Instruction::AddRegReg(_, _)
            | Instruction::SubRegReg(_, _)
            | Instruction::MulRegReg(_, _)
            | Instruction::AndRegReg(_, _)
            | Instruction::OrRegReg(_, _)
            | Instruction::LshRegReg(_, _)
            | Instruction::RshRegReg(_, _)
            | Instruction::XorRegReg(_, _) => InstructionKind::RegReg,

            Instruction::MovLitMem(_, _)
            | Instruction::JneLit(_, _)
            | Instruction::JeqLit(_, _)
            | Instruction::JgtLit(_, _)
            | Instruction::JgeLit(_, _)
            | Instruction::JleLit(_, _)
            | Instruction::JltLit(_, _) => InstructionKind::LitMem,

            Instruction::Inc(_)
            | Instruction::Dec(_)
            | Instruction::Not(_)
            | Instruction::PshReg(_)
            | Instruction::Pop(_) => InstructionKind::SingleReg,

            Instruction::MovRegMem(_, _)
            | Instruction::JneReg(_, _)
            | Instruction::JeqReg(_, _)
            | Instruction::JgtReg(_, _)
            | Instruction::JgeReg(_, _)
            | Instruction::JleReg(_, _)
            | Instruction::JltReg(_, _) => InstructionKind::RegMem,

            Instruction::MovMemReg(_, _) => InstructionKind::MemReg,
            Instruction::MovRegPtrReg(_, _) => InstructionKind::RegPtrReg,
            Instruction::PshLit(_) | Instruction::CallLit(_) | Instruction::Jmp(_) => InstructionKind::SingleLit,
            Instruction::Ret | Instruction::Hlt => InstructionKind::NoArgs,
        }
    }
}
