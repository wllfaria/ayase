use std::fmt;

use crate::instruction::{Instruction, InstructionSize};
use crate::memory::{self, Addressable};
use crate::op_code::{self, OpCode};
use crate::register::{self, Register, Registers};
use crate::word::Word;

#[derive(Debug)]
pub enum ExecutionFlow {
    Halt(u16),
    Continue,
}

#[derive(Debug)]
pub enum Error<const MEM_SIZE: usize> {
    Mem(memory::Error<MEM_SIZE>),
    OpCode(op_code::Error),
    Register(register::Error),
}

impl<const MEM_SIZE: usize> fmt::Display for Error<MEM_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<const MEM_SIZE: usize> std::error::Error for Error<MEM_SIZE> {}

impl<const MEM_SIZE: usize> From<memory::Error<MEM_SIZE>> for Error<MEM_SIZE> {
    fn from(err: memory::Error<MEM_SIZE>) -> Self {
        Self::Mem(err)
    }
}

impl<const MEM_SIZE: usize> From<op_code::Error> for Error<MEM_SIZE> {
    fn from(err: op_code::Error) -> Self {
        Self::OpCode(err)
    }
}

impl<const MEM_SIZE: usize> From<register::Error> for Error<MEM_SIZE> {
    fn from(err: register::Error) -> Self {
        Self::Register(err)
    }
}

type Result<const MEM_SIZE: usize, T> = std::result::Result<T, Error<MEM_SIZE>>;

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

    pub fn run(&mut self) {
        loop {
            match self.step() {
                Ok(ExecutionFlow::Halt(_)) => break,
                Ok(ExecutionFlow::Continue) => {}
                Err(e) => todo!("{e:?}"),
            }
        }
    }

    pub fn step(&mut self) -> Result<SIZE, ExecutionFlow> {
        let instruction = self.fetch()?;
        self.execute(instruction)
    }

    fn fetch(&mut self) -> Result<SIZE, Instruction<SIZE>> {
        let op = self.next_instruction(InstructionSize::Small)?;
        let op = OpCode::try_from(op)?;

        match op {
            OpCode::MovLitReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                let val = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::MovLitReg(reg, val))
            }
            OpCode::MovRegReg => {
                let reg_from = self.next_instruction(InstructionSize::Small)?;
                let reg_from = Register::try_from(reg_from)?;
                let reg_to = self.next_instruction(InstructionSize::Small)?;
                let reg_to = Register::try_from(reg_to)?;
                Ok(Instruction::MovRegReg(reg_from, reg_to))
            }
            OpCode::MovRegMem => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let address = self.next_instruction(InstructionSize::Word)?;
                let reg = Register::try_from(reg)?;
                let address = Word::try_from(address)?;
                Ok(Instruction::MovRegMem(reg, address))
            }
            OpCode::MovLitMem => {
                let val = self.next_instruction(InstructionSize::Word)?;
                let address = self.next_instruction(InstructionSize::Word)?;
                let address = Word::try_from(address)?;
                Ok(Instruction::MovLitMem(val, address))
            }
            OpCode::MovMemReg => {
                let address = self.next_instruction(InstructionSize::Word)?;
                let address = Word::try_from(address)?;
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::MovMemReg(address, reg))
            }
            OpCode::MovRegPtrReg => {
                let reg_from = self.next_instruction(InstructionSize::Small)?;
                let reg_to = self.next_instruction(InstructionSize::Small)?;
                let reg_from = Register::try_from(reg_from)?;
                let reg_to = Register::try_from(reg_to)?;
                // this register should hold a address, so we have to follow the pointer
                Ok(Instruction::MovRegPtrReg(reg_from, reg_to))
            }
            OpCode::PushLit => {
                let val = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::PushLit(val))
            }
            OpCode::PushReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                let val = self.registers.fetch(reg);
                Ok(Instruction::PushLit(val))
            }
            OpCode::PushRegPtr => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                // this register should hold a address, so we have to follow the pointer
                let val = self.registers.fetch(reg);
                let val = Word::try_from(val)?;
                let val = self.memory.read_word(val)?;
                Ok(Instruction::PushLit(val))
            }
            OpCode::Pop => Ok(Instruction::Pop),
            OpCode::PopReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::PopReg(reg))
            }
            OpCode::Call => {
                let word = self.next_instruction(InstructionSize::Word)?;
                let word = Word::try_from(word)?;
                Ok(Instruction::Call(word))
            }
            OpCode::Ret => Ok(Instruction::Ret),
            OpCode::Halt => {
                let code = self.next_instruction(InstructionSize::Small)?;
                Ok(Instruction::Halt(code))
            }
            OpCode::AddRegReg => todo!(),
            OpCode::AddLitReg => todo!(),
            OpCode::SubLitReg => todo!(),
            OpCode::SubRegLit => todo!(),
            OpCode::SubRegReg => todo!(),
            OpCode::IncReg => todo!(),
            OpCode::DecReg => todo!(),
            OpCode::MulLitReg => todo!(),
            OpCode::MulRegReg => todo!(),
            OpCode::LshLitReg => todo!(),
            OpCode::LshRegReg => todo!(),
            OpCode::RshLitReg => todo!(),
            OpCode::RshRegReg => todo!(),
            OpCode::AndLitReg => todo!(),
            OpCode::AndRegReg => todo!(),
            OpCode::OrLitReg => todo!(),
            OpCode::OrRegReg => todo!(),
            OpCode::XorLitReg => todo!(),
            OpCode::XorRegReg => todo!(),
            OpCode::Not => todo!(),
            OpCode::CallRegPtr => todo!(),
        }
    }

    fn execute(&mut self, instruction: Instruction<SIZE>) -> Result<SIZE, ExecutionFlow> {
        match instruction {
            Instruction::MovLitReg(reg, val) => self.registers.set(reg, val),
            Instruction::MovRegReg(from, to) => {
                let val = self.registers.fetch(from);
                self.registers.set(to, val);
            }
            Instruction::MovRegMem(reg, address) => {
                let val = self.registers.fetch(reg);
                self.memory.write_word(address, val)?;
            }
            Instruction::MovLitMem(val, address) => {
                self.memory.write_word(address, val)?;
            }
            Instruction::MovMemReg(address, reg) => {
                let value = self.memory.read_word(address)?;
                self.registers.set(reg, value)
            }
            Instruction::MovRegPtrReg(from, to) => {
                let val = self.registers.fetch(from);
                let val = Word::try_from(val)?;
                let val = self.memory.read_word(val)?;
                self.registers.set(to, val);
            }

            Instruction::PushLit(val) => self.push_stack(val)?,
            Instruction::Pop => _ = self.pop_stack()?,
            Instruction::PopReg(reg) => {
                let val = self.pop_stack()?;
                self.registers.set(reg, val);
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
            Instruction::Halt(code) => return Ok(ExecutionFlow::Halt(code)),
        }
        Ok(ExecutionFlow::Continue)
    }

    fn next_instruction(&mut self, size: InstructionSize) -> Result<SIZE, u16> {
        match size {
            InstructionSize::Small => {
                let reg_ptr = self.registers.fetch_word(Register::IP);
                let val = self.memory.read(reg_ptr)?;
                self.registers.set(Register::IP, reg_ptr.next()?.into());
                Ok(val.into())
            }
            InstructionSize::Word => {
                let reg_ptr = self.registers.fetch_word(Register::IP);
                let val = self.memory.read_word(reg_ptr)?;
                self.registers.set(Register::IP, reg_ptr.next_word()?.into());
                Ok(val)
            }
        }
    }

    fn pop_stack(&mut self) -> Result<SIZE, u16> {
        let stack_ptr = self.registers.fetch_word(Register::SP);
        let next = stack_ptr.next_word()?;
        let val = self.memory.read_word(next)?;
        self.registers.set(Register::SP, next.into());
        Ok(val)
    }

    fn push_stack(&mut self, val: u16) -> Result<SIZE, ()> {
        let stack_ptr = self.registers.fetch_word(Register::SP);
        self.memory.write_word(stack_ptr, val)?;
        self.registers.set(Register::SP, stack_ptr.prev_word()?.into());
        Ok(())
    }
}
