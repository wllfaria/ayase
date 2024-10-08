use crate::instruction::Instruction;
use crate::memory::{self, Addressable};
use crate::op_code::{self, OpCode};
use crate::register::{self, Register, Registers};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Mem(memory::Error),
    OpCode(op_code::Error),
    Register(register::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl From<memory::Error> for Error {
    fn from(err: memory::Error) -> Self {
        Self::Mem(err)
    }
}

impl From<op_code::Error> for Error {
    fn from(err: op_code::Error) -> Self {
        Self::OpCode(err)
    }
}

impl From<register::Error> for Error {
    fn from(err: register::Error) -> Self {
        Self::Register(err)
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Cpu<const SIZE: usize, A: Addressable<SIZE>> {
    pub registers: Registers<SIZE>,
    pub memory: A,
}

impl<const SIZE: usize, A: Addressable<SIZE>> Cpu<SIZE, A> {
    pub fn new(memory: A) -> Self {
        Self {
            registers: Registers::default(),
            memory,
        }
    }

    pub fn step(&mut self) -> Result<()> {
        let instruction = self.fetch()?;
        self.execute(instruction)
    }

    fn fetch(&mut self) -> Result<Instruction<SIZE>> {
        let inst_ptr = self.registers.fetch(Register::IP);
        let op = self.memory.read_word(inst_ptr)?;
        let op = OpCode::try_from(op)?;
        self.registers.set(Register::IP, inst_ptr.next_word());

        match op {
            OpCode::MovLitReg => {
                let reg_ptr = self.registers.fetch(Register::IP);
                let reg = self.memory.read_word(reg_ptr)?;
                let reg = Register::try_from(reg)?;
                self.registers.set(Register::IP, reg_ptr.next_word());
                let val_ptr = self.registers.fetch(Register::IP);
                let val = self.memory.read_word(val_ptr)?;
                self.registers.set(Register::IP, val_ptr.next_word());
                Ok(Instruction::MovToReg(reg, val.into()))
            }
        }
    }

    fn execute(&mut self, instruction: Instruction<SIZE>) -> Result<()> {
        match instruction {
            Instruction::MovToReg(reg, val) => self.registers.set(reg, val),
        }
        Ok(())
    }
}
