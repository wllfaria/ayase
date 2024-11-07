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
    MovRegReg       = 0x10,
    MovLitReg       = 0x11,
    MovRegMem       = 0x12,
    MovMemReg       = 0x13,
    MovLitMem       = 0x14,
    MovRegPtrReg    = 0x15,
    MovLitRegPtr    = 0x16,

    AddRegReg       = 0x20,
    AddLitReg       = 0x21,
    SubRegReg       = 0x22,
    SubLitReg       = 0x23,
    MulRegReg       = 0x24,
    MulLitReg       = 0x25,
    IncReg          = 0x26,
    DecReg          = 0x27,

    LshRegReg       = 0x30,
    LshLitReg       = 0x31,
    RshRegReg       = 0x32,
    RshLitReg       = 0x33,
    AndRegReg       = 0x34,
    AndLitReg       = 0x35,
    OrRegReg        = 0x36,
    OrLitReg        = 0x37,
    XorRegReg       = 0x38,
    XorLitReg       = 0x39,
    Not             = 0x3a,

    PushReg         = 0x40,
    PushLit         = 0x41,
    Pop             = 0x42,
    Call            = 0x43,
    Ret             = 0x44,

    JeqReg          = 0x51,
    JeqLit          = 0x52,
    JgtReg          = 0x53,
    JgtLit          = 0x54,
    JneReg          = 0x55,
    JneLit          = 0x56,
    JgeReg          = 0x57,
    JgeLit          = 0x58,
    JleReg          = 0x59,
    JleLit          = 0x5a,
    JltReg          = 0x5b,
    JltLit          = 0x5c,
    Jmp             = 0x5d,

    Int             = 0xfd,
    Rti             = 0xfe,
    Halt            = 0xff,
}
