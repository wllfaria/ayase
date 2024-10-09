#[derive(Debug, Clone, Copy)]
#[repr(u8)]
#[rustfmt::skip]
pub enum OpCode {
    MovLitReg   = 0x00,
    MovRegReg   = 0x01,

    PushLit     = 0x02,
    PushReg     = 0x03,
    PushRegPtr  = 0x04,
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
            v => Err(Error::InvalidValue(format!("value {v} is not a valid op code"))),
        }
    }
}
