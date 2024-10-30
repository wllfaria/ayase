use crate::parser::ast::ByteOffset;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
    pub kind: Kind,
    offset: ByteOffset,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            Kind::Ident => write!(f, "IDENT"),
            Kind::String => write!(f, "STRING"),
            Kind::HexNumber => write!(f, "HEX_NUMBER"),
            Kind::Const => write!(f, "CONST"),
            Kind::Data8 => write!(f, "DATA8"),
            Kind::Data16 => write!(f, "DATA16"),
            Kind::Import => write!(f, "IMPORT"),
            Kind::Bang => write!(f, "BANG"),
            Kind::LBracket => write!(f, "LEFT_BRACKET"),
            Kind::RBracket => write!(f, "RIGHT_BRACKET"),
            Kind::LParen => write!(f, "LEFT_PAREN"),
            Kind::RParen => write!(f, "RIGHT_PAREN"),
            Kind::LBrace => write!(f, "LEFT_CURLY"),
            Kind::RBrace => write!(f, "RIGHT_CURLY"),
            Kind::Equal => write!(f, "EQUAL"),
            Kind::Colon => write!(f, "COLON"),
            Kind::Comma => write!(f, "COMMA"),
            Kind::Ampersand => write!(f, "AMPERSAND"),
            Kind::Dot => write!(f, "DOT"),
            Kind::Mov => write!(f, "MOV"),
            Kind::Add => write!(f, "ADD"),
            Kind::Sub => write!(f, "SUB"),
            Kind::Mul => write!(f, "MUL"),
            Kind::Lsh => write!(f, "LSH"),
            Kind::Rsh => write!(f, "RSH"),
            Kind::And => write!(f, "AND"),
            Kind::Or => write!(f, "OR"),
            Kind::Xor => write!(f, "XOR"),
            Kind::Inc => write!(f, "INC"),
            Kind::Dec => write!(f, "DEC"),
            Kind::Not => write!(f, "NOT"),
            Kind::Jmp => write!(f, "JMP"),
            Kind::Jeq => write!(f, "JEQ"),
            Kind::Jgt => write!(f, "JGT"),
            Kind::Jne => write!(f, "JNE"),
            Kind::Jge => write!(f, "JGE"),
            Kind::Jle => write!(f, "JLE"),
            Kind::Jlt => write!(f, "JLT"),
            Kind::Psh => write!(f, "PSH"),
            Kind::Pop => write!(f, "POP"),
            Kind::Call => write!(f, "CALL"),
            Kind::Ret => write!(f, "RET"),
            Kind::Hlt => write!(f, "HLT"),
            Kind::Plus => write!(f, "PLUS"),
            Kind::Minus => write!(f, "MINUS"),
            Kind::Star => write!(f, "STAR"),
            Kind::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum Kind {
    Ident,
    String,
    HexNumber,

    Bang,
    Ampersand,
    LBracket,
    RBracket,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Colon,
    Comma,
    Dot,
    Equal,

    Const,
    Data8,
    Data16,
    Import,
    Mov,
    Add,
    Sub,
    Mul,
    Lsh,
    Rsh,
    And,
    Or,
    Xor,
    Inc,
    Dec,
    Not,
    Jmp,
    Jeq,
    Jgt,
    Jne,
    Jge,
    Jle,
    Jlt,
    Psh,
    Pop,
    Call,
    Ret,
    Hlt,

    Plus,
    Minus,
    Star,

    Eof,
}

impl Kind {
    pub fn is_instruction(&self) -> bool {
        match self {
            Kind::Const
            | Kind::Data8
            | Kind::Data16
            | Kind::Import
            | Kind::Ident
            | Kind::String
            | Kind::HexNumber
            | Kind::Bang
            | Kind::LBracket
            | Kind::RBracket
            | Kind::Ampersand
            | Kind::LParen
            | Kind::RParen
            | Kind::LBrace
            | Kind::RBrace
            | Kind::Colon
            | Kind::Comma
            | Kind::Equal
            | Kind::Dot
            | Kind::Plus
            | Kind::Minus
            | Kind::Star
            | Kind::Eof => false,
            Kind::Mov
            | Kind::Add
            | Kind::Sub
            | Kind::Mul
            | Kind::Lsh
            | Kind::Rsh
            | Kind::And
            | Kind::Or
            | Kind::Xor
            | Kind::Inc
            | Kind::Dec
            | Kind::Not
            | Kind::Jmp
            | Kind::Jeq
            | Kind::Jgt
            | Kind::Jne
            | Kind::Jge
            | Kind::Jle
            | Kind::Jlt
            | Kind::Psh
            | Kind::Pop
            | Kind::Call
            | Kind::Ret
            | Kind::Hlt => true,
        }
    }

    pub fn is_operator(&self) -> bool {
        match self {
            Kind::Plus | Kind::Minus | Kind::Star => true,
            Kind::Mov
            | Kind::Add
            | Kind::Sub
            | Kind::Eof
            | Kind::Mul
            | Kind::Lsh
            | Kind::Const
            | Kind::Data8
            | Kind::Data16
            | Kind::Import
            | Kind::Ident
            | Kind::String
            | Kind::HexNumber
            | Kind::Bang
            | Kind::LBracket
            | Kind::RBracket
            | Kind::Ampersand
            | Kind::LParen
            | Kind::RParen
            | Kind::LBrace
            | Kind::RBrace
            | Kind::Colon
            | Kind::Comma
            | Kind::Equal
            | Kind::Dot
            | Kind::Rsh
            | Kind::And
            | Kind::Or
            | Kind::Xor
            | Kind::Inc
            | Kind::Dec
            | Kind::Not
            | Kind::Jmp
            | Kind::Jeq
            | Kind::Jgt
            | Kind::Jne
            | Kind::Jge
            | Kind::Jle
            | Kind::Jlt
            | Kind::Psh
            | Kind::Pop
            | Kind::Call
            | Kind::Ret
            | Kind::Hlt => true,
        }
    }
}

impl Token {
    pub fn new(kind: Kind, offset: impl Into<ByteOffset>) -> Self {
        Self {
            offset: offset.into(),
            kind,
        }
    }

    pub fn from_ident(ident: &str, start: usize, end: usize) -> Token {
        match ident.to_lowercase().as_str() {
            "const" => Token {
                offset: (start..end).into(),
                kind: Kind::Const,
            },
            "import" => Token {
                offset: (start..end).into(),
                kind: Kind::Import,
            },
            "data8" => Token {
                offset: (start..end).into(),
                kind: Kind::Data8,
            },
            "data16" => Token {
                offset: (start..end).into(),
                kind: Kind::Data16,
            },
            "mov" => Token {
                offset: (start..end).into(),
                kind: Kind::Mov,
            },
            "add" => Token {
                offset: (start..end).into(),
                kind: Kind::Add,
            },
            "sub" => Token {
                offset: (start..end).into(),
                kind: Kind::Sub,
            },
            "mul" => Token {
                offset: (start..end).into(),
                kind: Kind::Mul,
            },
            "lsh" => Token {
                offset: (start..end).into(),
                kind: Kind::Lsh,
            },
            "rsh" => Token {
                offset: (start..end).into(),
                kind: Kind::Rsh,
            },
            "and" => Token {
                offset: (start..end).into(),
                kind: Kind::And,
            },
            "or" => Token {
                offset: (start..end).into(),
                kind: Kind::Or,
            },
            "xor" => Token {
                offset: (start..end).into(),
                kind: Kind::Xor,
            },
            "inc" => Token {
                offset: (start..end).into(),
                kind: Kind::Inc,
            },
            "dec" => Token {
                offset: (start..end).into(),
                kind: Kind::Dec,
            },
            "not" => Token {
                offset: (start..end).into(),
                kind: Kind::Not,
            },
            "jmp" => Token {
                offset: (start..end).into(),
                kind: Kind::Jmp,
            },
            "jeq" => Token {
                offset: (start..end).into(),
                kind: Kind::Jeq,
            },
            "jgt" => Token {
                offset: (start..end).into(),
                kind: Kind::Jgt,
            },
            "jne" => Token {
                offset: (start..end).into(),
                kind: Kind::Jne,
            },
            "jge" => Token {
                offset: (start..end).into(),
                kind: Kind::Jge,
            },
            "jle" => Token {
                offset: (start..end).into(),
                kind: Kind::Jle,
            },
            "jlt" => Token {
                offset: (start..end).into(),
                kind: Kind::Jlt,
            },
            "psh" => Token {
                offset: (start..end).into(),
                kind: Kind::Psh,
            },
            "pop" => Token {
                offset: (start..end).into(),
                kind: Kind::Pop,
            },
            "call" => Token {
                offset: (start..end).into(),
                kind: Kind::Call,
            },
            "ret" => Token {
                offset: (start..end).into(),
                kind: Kind::Ret,
            },
            "hlt" => Token {
                offset: (start..end).into(),
                kind: Kind::Hlt,
            },
            _ => Token {
                offset: (start..end).into(),
                kind: Kind::Ident,
            },
        }
    }

    pub fn offset(&self) -> ByteOffset {
        self.offset
    }
}
