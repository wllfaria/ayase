#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    MovLitReg = 0x00,
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
            v => Err(Error::InvalidValue(format!(
                "value {v} is not a valid op code"
            ))),
        }
    }
}
