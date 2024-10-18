use aya_core::op_code::OpCode;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mul,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Ast<'parser> {
    Instruction(Box<Instruction<'parser>>),
    HexLiteral(&'parser str),
    Address(&'parser str),
    Register(&'parser str),
    Var(&'parser str),
    Operator(Operator),
    Label(&'parser str),
    FieldAccessor {
        module: &'parser str,
        field: &'parser str,
    },
    Import {
        name: &'parser str,
        path: &'parser str,
        address: Box<Ast<'parser>>,
        variables: Vec<Ast<'parser>>,
    },
    ImportVar {
        name: &'parser str,
        value: Box<Ast<'parser>>,
    },
    Data {
        name: &'parser str,
        size: u8,
        exported: bool,
        values: Vec<Ast<'parser>>,
    },
    Const {
        name: &'parser str,
        exported: bool,
        value: Box<Ast<'parser>>,
    },
    BinaryOp {
        lhs: Box<Ast<'parser>>,
        operator: Operator,
        rhs: Box<Ast<'parser>>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum InstructionKind {
    LitReg,
    RegLit,
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
            InstructionKind::RegLit => 4,
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
pub enum Instruction<'parser> {
    MovLitReg(Ast<'parser>, Ast<'parser>),
    MovRegReg(Ast<'parser>, Ast<'parser>),
    MovRegMem(Ast<'parser>, Ast<'parser>),
    MovMemReg(Ast<'parser>, Ast<'parser>),
    MovLitMem(Ast<'parser>, Ast<'parser>),
    MovRegPtrReg(Ast<'parser>, Ast<'parser>),
    AddRegReg(Ast<'parser>, Ast<'parser>),
    AddLitReg(Ast<'parser>, Ast<'parser>),
    SubRegReg(Ast<'parser>, Ast<'parser>),
    SubLitReg(Ast<'parser>, Ast<'parser>),
    MulRegReg(Ast<'parser>, Ast<'parser>),
    MulLitReg(Ast<'parser>, Ast<'parser>),
    LshRegReg(Ast<'parser>, Ast<'parser>),
    LshLitReg(Ast<'parser>, Ast<'parser>),
    RshRegReg(Ast<'parser>, Ast<'parser>),
    RshLitReg(Ast<'parser>, Ast<'parser>),
    AndRegReg(Ast<'parser>, Ast<'parser>),
    AndLitReg(Ast<'parser>, Ast<'parser>),
    OrLitReg(Ast<'parser>, Ast<'parser>),
    OrRegReg(Ast<'parser>, Ast<'parser>),
    XorLitReg(Ast<'parser>, Ast<'parser>),
    XorRegReg(Ast<'parser>, Ast<'parser>),
    Inc(Ast<'parser>),
    Dec(Ast<'parser>),
    Not(Ast<'parser>),
    JeqLit(Ast<'parser>, Ast<'parser>),
    JeqReg(Ast<'parser>, Ast<'parser>),
    JgtLit(Ast<'parser>, Ast<'parser>),
    JgtReg(Ast<'parser>, Ast<'parser>),
    JneLit(Ast<'parser>, Ast<'parser>),
    JneReg(Ast<'parser>, Ast<'parser>),
    JgeLit(Ast<'parser>, Ast<'parser>),
    JgeReg(Ast<'parser>, Ast<'parser>),
    JleLit(Ast<'parser>, Ast<'parser>),
    JleReg(Ast<'parser>, Ast<'parser>),
    JltLit(Ast<'parser>, Ast<'parser>),
    JltReg(Ast<'parser>, Ast<'parser>),
    PshLit(Ast<'parser>),
    PshReg(Ast<'parser>),
    Pop(Ast<'parser>),
    CalLit(Ast<'parser>),
    CalReg(Ast<'parser>),
    Ret,
    Hlt,
}

impl<'parser> Instruction<'parser> {
    pub fn lhs(&self) -> &Ast<'parser> {
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
            | Instruction::CalLit(lhs)
            | Instruction::CalReg(lhs)
            | Instruction::Inc(lhs)
            | Instruction::Dec(lhs)
            | Instruction::Not(lhs) => lhs,

            Instruction::Ret | Instruction::Hlt => unreachable!(),
        }
    }

    pub fn rhs(&self) -> &Ast<'parser> {
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
            | Instruction::CalLit(_)
            | Instruction::CalReg(_)
            | Instruction::Inc(_)
            | Instruction::Dec(_)
            | Instruction::Not(_)
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
            Instruction::CalLit(_) => OpCode::Call,
            Instruction::CalReg(_) => OpCode::CallRegPtr,
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
            | Instruction::Pop(_)
            | Instruction::CalReg(_) => InstructionKind::SingleReg,

            Instruction::MovRegMem(_, _)
            | Instruction::JneReg(_, _)
            | Instruction::JeqReg(_, _)
            | Instruction::JgtReg(_, _)
            | Instruction::JgeReg(_, _)
            | Instruction::JleReg(_, _)
            | Instruction::JltReg(_, _) => InstructionKind::RegMem,

            Instruction::MovMemReg(_, _) => InstructionKind::MemReg,
            Instruction::MovRegPtrReg(_, _) => InstructionKind::RegPtrReg,
            Instruction::PshLit(_) | Instruction::CalLit(_) => InstructionKind::SingleLit,
            Instruction::Ret | Instruction::Hlt => InstructionKind::NoArgs,
        }
    }
}
