#[derive(Debug, Clone, Copy)]
#[repr(u8)]
#[rustfmt::skip]
pub enum OpCode {
    MovLitReg   = 0x00,
    MovRegReg   = 0x01,

    PushLit     = 0x02,
    PushReg     = 0x03,
    PushRegPtr  = 0x04,

    PopReg      = 0x05,
    Pop         = 0x06,

    Call        = 0x10,
    Ret         = 0x11,

    Halt        = 0xff,
}

#[derive(Debug)]
pub enum Error {
    InvalidValue(String),
}

type Result = std::result::Result<OpCode, Error>;

impl TryFrom<u16> for OpCode {
    type Error = Error;

    fn try_from(value: u16) -> Result {
        match value {
            v if v == OpCode::MovLitReg as u16 => Ok(OpCode::MovLitReg),
            v if v == OpCode::MovRegReg as u16 => Ok(OpCode::MovRegReg),
            v if v == OpCode::PushLit as u16 => Ok(OpCode::PushLit),
            v if v == OpCode::PushReg as u16 => Ok(OpCode::PushReg),
            v if v == OpCode::PushRegPtr as u16 => Ok(OpCode::PushRegPtr),
            v if v == OpCode::PopReg as u16 => Ok(OpCode::PopReg),
            v if v == OpCode::Pop as u16 => Ok(OpCode::Pop),
            v if v == OpCode::Call as u16 => Ok(OpCode::Call),
            v if v == OpCode::Ret as u16 => Ok(OpCode::Ret),
            v if v == OpCode::Halt as u16 => Ok(OpCode::Halt),
            v => Err(Error::InvalidValue(format!("value {v} is not a valid op code"))),
        }
    }
}
