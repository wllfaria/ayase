#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mul,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Atom<'parser> {
    HexLiteral(&'parser str),
    Address(&'parser str),
    Register(&'parser str),
    Var(&'parser str),
    Operator(Operator),
    BinaryOp {
        lhs: Box<Atom<'parser>>,
        operator: Operator,
        rhs: Box<Atom<'parser>>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction<'parser> {
    MovLitReg(Atom<'parser>, Atom<'parser>),
    MovRegReg(Atom<'parser>, Atom<'parser>),
    MovRegMem(Atom<'parser>, Atom<'parser>),
    MovMemReg(Atom<'parser>, Atom<'parser>),
    MovLitMem(Atom<'parser>, Atom<'parser>),
    MovRegPtrReg(Atom<'parser>, Atom<'parser>),
}
