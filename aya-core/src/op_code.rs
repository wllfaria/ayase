#[derive(Debug)]
pub enum Error {
    InvalidValue(String),
}

type Result = std::result::Result<OpCode, Error>;

macro_rules! op_codes {
    ($($variant:ident = $value:expr),* $(,)?) => {
        #[derive(Debug, Clone, Copy)]
        #[repr(u8)]
        #[rustfmt::skip]
        pub enum OpCode {
            $($variant = $value),*
        }

        impl TryFrom<u16> for OpCode {
            type Error = Error;

            fn try_from(value: u16) -> Result {
                match value {
                    $(x if x == $value => Ok(OpCode::$variant),)*
                    v => Err(Error::InvalidValue(format!("value {v} is not a valid op code"))),
                }
            }
        }

        impl From<OpCode> for u8 {
            fn from(opcode: OpCode) -> Self  {
                opcode as u8
            }
        }
    }
}

op_codes! {
    MovLitReg       = 0x10,
    MovRegReg       = 0x11,
    MovRegMem       = 0x12,
    MovMemReg       = 0x13,
    MovLitMem       = 0x14,
    MovRegPtrReg    = 0x15,

    AddRegReg       = 0x20,
    AddLitReg       = 0x21,
    SubRegReg       = 0x24,
    SubLitReg       = 0x22,
    IncReg          = 0x25,
    DecReg          = 0x26,
    MulLitReg       = 0x27,
    MulRegReg       = 0x28,

    LshLitReg       = 0x30,
    LshRegReg       = 0x31,
    RshLitReg       = 0x32,
    RshRegReg       = 0x33,
    AndLitReg       = 0x34,
    AndRegReg       = 0x35,
    OrLitReg        = 0x36,
    OrRegReg        = 0x37,
    XorLitReg       = 0x38,
    XorRegReg       = 0x39,
    Not             = 0x3a,

    PushLit         = 0x40,
    PushReg         = 0x41,
    Pop             = 0x42,
    Call            = 0x43,
    CallRegPtr      = 0x44,
    Ret             = 0x45,

    JeqLit          = 0x51,
    JeqReg          = 0x52,
    JgtLit          = 0x53,
    JgtReg          = 0x54,
    JneLit          = 0x55,
    JneReg          = 0x56,
    JgeLit          = 0x57,
    JgeReg          = 0x58,
    JleLit          = 0x59,
    JleReg          = 0x5a,
    JltLit          = 0x5b,
    JltReg          = 0x5c,

    Halt            = 0xff,
}
