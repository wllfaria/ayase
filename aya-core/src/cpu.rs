use std::fmt;

use crate::instruction::Instruction;
use crate::memory::{self, Addressable};
use crate::op_code::{self, OpCode};
use crate::register::{self, Register, Registers};
use crate::word::Word;

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
        println!("{instruction:?}");
        self.execute(instruction)
    }

    fn fetch(&mut self) -> Result<Instruction<SIZE>> {
        let inst_ptr = self.registers.fetch_word(Register::IP);
        let op = self.memory.read_word(inst_ptr)?;
        let op = OpCode::try_from(op)?;
        self.registers.set(Register::IP, inst_ptr.next_word()?.into());

        match op {
            OpCode::MovLitReg => {
                let reg_ptr = self.registers.fetch_word(Register::IP);
                let reg = self.memory.read_word(reg_ptr)?;
                let reg = Register::try_from(reg)?;
                self.registers.set(Register::IP, reg_ptr.next_word()?.into());
                let val_ptr = self.registers.fetch_word(Register::IP);
                let val = self.memory.read_word(val_ptr)?;
                self.registers.set(Register::IP, val_ptr.next_word()?.into());
                Ok(Instruction::MovLitReg(reg, val))
            }
            OpCode::MovRegReg => {
                let reg_ptr = self.registers.fetch_word(Register::IP);
                let reg_from = self.memory.read_word(reg_ptr)?;
                let reg_from = Register::try_from(reg_from)?;
                self.registers.set(Register::IP, reg_ptr.next_word()?.into());
                let reg_ptr = self.registers.fetch_word(Register::IP);
                let reg_to = self.memory.read_word(reg_ptr)?;
                let reg_to = Register::try_from(reg_to)?;
                self.registers.set(Register::IP, reg_ptr.next_word()?.into());
                Ok(Instruction::MovRegReg(reg_from, reg_to))
            }
            OpCode::PushLit => {
                let reg_ptr = self.registers.fetch_word(Register::IP);
                let val = self.memory.read_word(reg_ptr)?;
                self.registers.set(Register::IP, reg_ptr.next_word()?.into());
                let stack_ptr = self.registers.fetch_word(Register::SP);
                self.registers.set(Register::SP, stack_ptr.prev_word()?.into());
                Ok(Instruction::PushLit(stack_ptr, val))
            }
            OpCode::PushReg => {
                let reg_ptr = self.registers.fetch_word(Register::IP);
                let reg = self.memory.read_word(reg_ptr)?;
                let reg = Register::try_from(reg)?;
                let val = self.registers.fetch(reg);
                self.registers.set(Register::IP, reg_ptr.next_word()?.into());
                let stack_ptr = self.registers.fetch_word(Register::SP);
                self.registers.set(Register::SP, stack_ptr.prev_word()?.into());
                Ok(Instruction::PushLit(stack_ptr, val))
            }
            OpCode::PushRegPtr => {
                let reg_ptr = self.registers.fetch_word(Register::IP);
                let reg = self.memory.read_word(reg_ptr)?;
                let reg = Register::try_from(reg)?;
                self.registers.set(Register::IP, reg_ptr.next_word()?.into());
                // this register should hold a address, so we have to follow the pointer
                let val = self.registers.fetch(reg);
                let val = Word::try_from(val)?;
                let val = self.memory.read_word(val)?;
                let stack_ptr = self.registers.fetch_word(Register::SP);
                self.registers.set(Register::SP, stack_ptr.prev_word()?.into());
                Ok(Instruction::PushLit(stack_ptr, val))
            }
        }
    }

    fn execute(&mut self, instruction: Instruction<SIZE>) -> Result<()> {
        match instruction {
            Instruction::MovLitReg(reg, val) => self.registers.set(reg, val),
            Instruction::MovRegReg(from, to) => {
                let val = self.registers.fetch(from);
                self.registers.set(to, val);
            }
            Instruction::PushLit(address, val) => {
                self.memory.write_word(address, val)?;
            }
        }
        Ok(())
    }
}
