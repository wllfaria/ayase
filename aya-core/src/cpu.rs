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
        #[cfg(debug_assertions)]
        println!("{instruction:?}");
        self.execute(instruction)
    }

    fn fetch(&mut self) -> Result<Instruction<SIZE>> {
        let op = self.next_instruction()?;
        let op = OpCode::try_from(op)?;

        match op {
            OpCode::MovLitReg => {
                let reg = self.next_instruction()?;
                let reg = Register::try_from(reg)?;
                let val_ptr = self.registers.fetch_word(Register::IP);
                let val = self.memory.read_word(val_ptr)?;
                self.registers.set(Register::IP, val_ptr.next_word()?.into());
                Ok(Instruction::MovLitReg(reg, val))
            }
            OpCode::MovRegReg => {
                let reg_from = self.next_instruction()?;
                let reg_from = Register::try_from(reg_from)?;
                let reg_ptr = self.registers.fetch_word(Register::IP);
                let reg_to = self.memory.read_word(reg_ptr)?;
                let reg_to = Register::try_from(reg_to)?;
                self.registers.set(Register::IP, reg_ptr.next_word()?.into());
                Ok(Instruction::MovRegReg(reg_from, reg_to))
            }
            OpCode::PushLit => {
                let val = self.next_instruction()?;
                let stack_ptr = self.registers.fetch_word(Register::SP);
                self.registers.set(Register::SP, stack_ptr.prev_word()?.into());
                Ok(Instruction::PushLit(stack_ptr, val))
            }
            OpCode::PushReg => {
                let reg = self.next_instruction()?;
                let reg = Register::try_from(reg)?;
                let val = self.registers.fetch(reg);
                let stack_ptr = self.registers.fetch_word(Register::SP);
                self.registers.set(Register::SP, stack_ptr.prev_word()?.into());
                Ok(Instruction::PushLit(stack_ptr, val))
            }
            OpCode::PushRegPtr => {
                let reg = self.next_instruction()?;
                let reg = Register::try_from(reg)?;
                // this register should hold a address, so we have to follow the pointer
                let val = self.registers.fetch(reg);
                let val = Word::try_from(val)?;
                let val = self.memory.read_word(val)?;
                let stack_ptr = self.registers.fetch_word(Register::SP);
                self.registers.set(Register::SP, stack_ptr.prev_word()?.into());
                Ok(Instruction::PushLit(stack_ptr, val))
            }
            OpCode::Pop => {
                let val = self.pop_stack()?;
                Ok(Instruction::Pop(None, val))
            }
            OpCode::PopReg => {
                let reg = self.next_instruction()?;
                let reg = Register::try_from(reg)?;
                let val = self.pop_stack()?;
                Ok(Instruction::Pop(Some(reg), val))
            }
            OpCode::Call => {
                let word = self.next_instruction()?;
                let word = Word::try_from(word)?;
                Ok(Instruction::Call(word))
            }
            OpCode::Ret => Ok(Instruction::Ret),
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
            Instruction::Pop(reg, val) => {
                if let Some(reg) = reg {
                    self.registers.set(reg, val);
                }
            }
            Instruction::Call(address) => {
                // when calling a subroutine, we need to finish the current stack frame by:
                // 1. pushing the state of every non volatile general purpose register (R1 to R4)
                // 2. pushing the current address of the instruction pointer
                // 3. pushing the size of the current stack frame.
                // 4. moving the stack and frame pointer to the next address
                let r1 = self.registers.fetch(Register::R1);
                let r2 = self.registers.fetch(Register::R2);
                let r3 = self.registers.fetch(Register::R3);
                let r4 = self.registers.fetch(Register::R4);
                let ip = self.registers.fetch(Register::IP);

                self.push_stack(r1)?;
                self.push_stack(r2)?;
                self.push_stack(r3)?;
                self.push_stack(r4)?;
                self.push_stack(ip)?;

                let stack_ptr = self.registers.fetch_word(Register::SP);
                let frame_ptr = self.registers.fetch_word(Register::FP);
                let next_frame_start = stack_ptr.prev_word()?;
                let frame_size = frame_ptr - next_frame_start;
                self.memory.write_word(stack_ptr, frame_size.into())?;
                self.registers.set(Register::SP, next_frame_start.into());
                self.registers.set(Register::FP, next_frame_start.into());
                self.registers.set(Register::IP, address.into());
            }
            Instruction::Ret => {
                // when returning from a subroutine, we need to restore registers to same state as
                // they were before calling this subroutine by:
                // 1. moving the frame pointer back to the beginning of the previous stack frame
                // 2. moving the stack pointer to the previous instruction pointer address
                // 3. restoring the values of the non volatile registers (R1-R4)

                let frame_ptr = self.registers.fetch_word(Register::FP);
                // we set the stack pointer back to the frame pointer to pop the previous values
                self.registers.set(Register::SP, frame_ptr.into());

                let frame_size = self.pop_stack()?;
                let ip = self.pop_stack()?;
                let r4 = self.pop_stack()?;
                let r3 = self.pop_stack()?;
                let r2 = self.pop_stack()?;
                let r1 = self.pop_stack()?;

                self.registers.set(Register::IP, ip);
                self.registers.set(Register::R4, r4);
                self.registers.set(Register::R3, r3);
                self.registers.set(Register::R2, r2);
                self.registers.set(Register::R1, r1);

                let prev_frame_ptr = frame_ptr + Word::try_from(frame_size)?;
                self.registers.set(Register::FP, prev_frame_ptr.into());
            }
        }
        Ok(())
    }

    fn next_instruction(&mut self) -> Result<u16> {
        let reg_ptr = self.registers.fetch_word(Register::IP);
        let val = self.memory.read_word(reg_ptr)?;
        self.registers.set(Register::IP, reg_ptr.next_word()?.into());
        Ok(val)
    }

    fn pop_stack(&mut self) -> Result<u16> {
        let stack_ptr = self.registers.fetch_word(Register::SP);
        let next = stack_ptr.next_word()?;
        let val = self.memory.read_word(next)?;
        self.registers.set(Register::SP, next.into());
        Ok(val)
    }

    fn push_stack(&mut self, val: u16) -> Result<()> {
        let stack_ptr = self.registers.fetch_word(Register::SP);
        self.memory.write_word(stack_ptr, val)?;
        self.registers.set(Register::SP, stack_ptr.prev_word()?.into());
        Ok(())
    }
}
