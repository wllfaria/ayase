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
    SubLitReg       = 0x22,
    SubRegLit       = 0x23,
    SubRegReg       = 0x24,
    IncReg          = 0x25,
    DecReg          = 0x26,
    MulLitReg       = 0x27,
    MulRegReg       = 0x28,

    LsfLitReg       = 0x30,
    LsfRegReg       = 0x31,
    RsfLitReg       = 0x32,
    RsfRegReg       = 0x33,
    AndLitReg       = 0x34,
    AndRegReg       = 0x35,
    OrLitReg        = 0x36,
    OrRegReg        = 0x37,
    XorLitReg       = 0x38,
    XorRegReg       = 0x39,
    Not             = 0x3a,

    PushLit         = 0x40,
    PushReg         = 0x41,
    PushRegPtr      = 0x42,
    Pop             = 0x43,
    PopReg          = 0x44,
    Call            = 0x45,
    CallRegPtr      = 0x46,
    Ret             = 0x47,

    Halt            = 0xff,
}
