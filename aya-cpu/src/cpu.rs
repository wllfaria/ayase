use crate::error::Result;
use crate::instruction::{Instruction, InstructionSize};
use crate::memory::Addressable;
use crate::op_code::OpCode;
use crate::register::{Register, Registers};
use crate::word::Word;

#[derive(Debug)]
pub enum ControlFlow {
    Halt(u16),
    Continue,
}

#[derive(Debug)]
pub struct Cpu<A: Addressable> {
    pub registers: Registers,
    pub memory: A,
    start_address: Word,
    in_interrupt: bool,
    interrupt_table: Word,
}

impl<A: Addressable> Cpu<A> {
    pub fn new<W>(memory: A, start_address: W, stack_address: W, interrupt_table: W) -> Self
    where
        W: Into<Word> + Copy,
    {
        Self {
            registers: Registers::new(start_address, stack_address),
            memory,
            start_address: start_address.into(),
            in_interrupt: false,
            interrupt_table: interrupt_table.into(),
        }
    }

    pub fn load_into_address(&mut self, bytecode: impl AsRef<[u8]>, address: impl TryInto<Word>) -> Result<()> {
        let mut address = match address.try_into() {
            Ok(addr) => addr,
            Err(_) => unreachable!(),
        };
        for byte in bytecode.as_ref() {
            self.memory.write(address, *byte)?;
            address = address.next()?;
        }
        Ok(())
    }

    pub fn run(&mut self) {
        loop {
            match self.step() {
                Ok(ControlFlow::Halt(_)) => break,
                Ok(ControlFlow::Continue) => {}
                Err(e) => todo!("{e:?}"),
            }
        }
    }

    pub fn step(&mut self) -> Result<ControlFlow> {
        let instruction = self.fetch()?;
        self.execute(instruction)
    }

    fn fetch(&mut self) -> Result<Instruction> {
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
                let reg_to = self.next_instruction(InstructionSize::Small)?;
                let reg_to = Register::try_from(reg_to)?;
                let reg_from = self.next_instruction(InstructionSize::Small)?;
                let reg_from = Register::try_from(reg_from)?;
                Ok(Instruction::MovRegReg(reg_to, reg_from))
            }
            OpCode::MovRegMem => {
                let address = self.next_instruction(InstructionSize::Word)?;
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::MovRegMem(reg, address.into()))
            }
            OpCode::MovLitMem => {
                let address = self.next_instruction(InstructionSize::Word)?;
                let val = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::MovLitMem(address.into(), val))
            }
            OpCode::MovMemReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                let address = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::MovMemReg(address.into(), reg))
            }
            OpCode::MovRegPtrReg => {
                let reg_to = self.next_instruction(InstructionSize::Small)?;
                let reg_to = Register::try_from(reg_to)?;
                let reg_from = self.next_instruction(InstructionSize::Small)?;
                let reg_from = Register::try_from(reg_from)?;
                Ok(Instruction::MovRegPtrReg(reg_to, reg_from))
            }
            OpCode::MovLitRegPtr => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let lit = self.next_instruction(InstructionSize::Word)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::MovLitRegPtr(reg, lit))
            }
            OpCode::Mov8LitReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                let val = self.next_instruction(InstructionSize::Small)?;
                let val = (val & 0xFF) as u8;
                Ok(Instruction::Mov8LitReg(reg, val))
            }
            OpCode::Mov8RegReg => {
                let reg_from = self.next_instruction(InstructionSize::Small)?;
                let reg_from = Register::try_from(reg_from)?;
                let reg_to = self.next_instruction(InstructionSize::Small)?;
                let reg_to = Register::try_from(reg_to)?;
                Ok(Instruction::Mov8RegReg(reg_from, reg_to))
            }
            OpCode::Mov8RegMem => {
                let address = self.next_instruction(InstructionSize::Word)?;
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::Mov8RegMem(reg, address.into()))
            }
            OpCode::Mov8MemReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                let address = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::Mov8MemReg(address.into(), reg))
            }
            OpCode::Mov8LitMem => {
                let address = self.next_instruction(InstructionSize::Word)?;
                let val = self.next_instruction(InstructionSize::Small)?;
                let val = (val & 0xFF) as u8;
                Ok(Instruction::Mov8LitMem(address.into(), val))
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
            OpCode::Pop => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::PopReg(reg))
            }
            OpCode::Call => {
                let word = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::Call(word.into()))
            }
            OpCode::Ret => Ok(Instruction::Ret),
            OpCode::Halt => {
                let code = self.next_instruction(InstructionSize::Small)?;
                Ok(Instruction::Halt(code))
            }
            OpCode::AddRegReg => {
                let r1 = self.next_instruction(InstructionSize::Small)?;
                let r1 = Register::try_from(r1)?;
                let r2 = self.next_instruction(InstructionSize::Small)?;
                let r2 = Register::try_from(r2)?;
                Ok(Instruction::AddRegReg(r1, r2))
            }
            OpCode::AddLitReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                let lit = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::AddLitReg(reg, lit))
            }
            OpCode::SubLitReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                let lit = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::SubLitReg(reg, lit))
            }
            OpCode::SubRegReg => {
                let r1 = self.next_instruction(InstructionSize::Small)?;
                let r1 = Register::try_from(r1)?;
                let r2 = self.next_instruction(InstructionSize::Small)?;
                let r2 = Register::try_from(r2)?;
                Ok(Instruction::SubRegReg(r1, r2))
            }
            OpCode::IncReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::IncReg(reg))
            }
            OpCode::DecReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::DecReg(reg))
            }
            OpCode::MulLitReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                let lit = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::MulLitReg(reg, lit))
            }
            OpCode::MulRegReg => {
                let r1 = self.next_instruction(InstructionSize::Small)?;
                let r1 = Register::try_from(r1)?;
                let r2 = self.next_instruction(InstructionSize::Small)?;
                let r2 = Register::try_from(r2)?;
                Ok(Instruction::MulRegReg(r1, r2))
            }

            OpCode::LshLitReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                let lit = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::LshLitReg(reg, lit))
            }
            OpCode::LshRegReg => {
                let r1 = self.next_instruction(InstructionSize::Small)?;
                let r1 = Register::try_from(r1)?;
                let r2 = self.next_instruction(InstructionSize::Small)?;
                let r2 = Register::try_from(r2)?;
                Ok(Instruction::LshRegReg(r1, r2))
            }
            OpCode::RshLitReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                let lit = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::RshLitReg(reg, lit))
            }
            OpCode::RshRegReg => {
                let r1 = self.next_instruction(InstructionSize::Small)?;
                let r1 = Register::try_from(r1)?;
                let r2 = self.next_instruction(InstructionSize::Small)?;
                let r2 = Register::try_from(r2)?;
                Ok(Instruction::RshRegReg(r1, r2))
            }
            OpCode::AndLitReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                let lit = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::AndLitReg(reg, lit))
            }
            OpCode::AndRegReg => {
                let r1 = self.next_instruction(InstructionSize::Small)?;
                let r1 = Register::try_from(r1)?;
                let r2 = self.next_instruction(InstructionSize::Small)?;
                let r2 = Register::try_from(r2)?;
                Ok(Instruction::AndRegReg(r1, r2))
            }
            OpCode::OrLitReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                let lit = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::OrLitReg(reg, lit))
            }
            OpCode::OrRegReg => {
                let r1 = self.next_instruction(InstructionSize::Small)?;
                let r1 = Register::try_from(r1)?;
                let r2 = self.next_instruction(InstructionSize::Small)?;
                let r2 = Register::try_from(r2)?;
                Ok(Instruction::OrRegReg(r1, r2))
            }
            OpCode::XorLitReg => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                let lit = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::XorLitReg(reg, lit))
            }
            OpCode::XorRegReg => {
                let r1 = self.next_instruction(InstructionSize::Small)?;
                let r1 = Register::try_from(r1)?;
                let r2 = self.next_instruction(InstructionSize::Small)?;
                let r2 = Register::try_from(r2)?;
                Ok(Instruction::XorRegReg(r1, r2))
            }
            OpCode::Not => {
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::Not(reg))
            }

            OpCode::JeqLit => {
                let jump_to = self.next_instruction(InstructionSize::Word)?;
                let literal = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::JeqLit(jump_to.into(), literal))
            }
            OpCode::JeqReg => {
                let jump_to = self.next_instruction(InstructionSize::Word)?;
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::JeqReg(jump_to.into(), reg))
            }
            OpCode::JgtLit => {
                let jump_to = self.next_instruction(InstructionSize::Word)?;
                let literal = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::JgtLit(jump_to.into(), literal))
            }
            OpCode::JgtReg => {
                let jump_to = self.next_instruction(InstructionSize::Word)?;
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::JgtReg(jump_to.into(), reg))
            }
            OpCode::JneLit => {
                let jump_to = self.next_instruction(InstructionSize::Word)?;
                let literal = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::JneLit(jump_to.into(), literal))
            }
            OpCode::JneReg => {
                let jump_to = self.next_instruction(InstructionSize::Word)?;
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::JneReg(jump_to.into(), reg))
            }
            OpCode::JgeLit => {
                let jump_to = self.next_instruction(InstructionSize::Word)?;
                let literal = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::JgeLit(jump_to.into(), literal))
            }
            OpCode::JgeReg => {
                let jump_to = self.next_instruction(InstructionSize::Word)?;
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::JgeReg(jump_to.into(), reg))
            }
            OpCode::JleLit => {
                let jump_to = self.next_instruction(InstructionSize::Word)?;
                let literal = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::JleLit(jump_to.into(), literal))
            }
            OpCode::JleReg => {
                let jump_to = self.next_instruction(InstructionSize::Word)?;
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::JleReg(jump_to.into(), reg))
            }
            OpCode::JltLit => {
                let jump_to = self.next_instruction(InstructionSize::Word)?;
                let literal = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::JltLit(jump_to.into(), literal))
            }
            OpCode::JltReg => {
                let jump_to = self.next_instruction(InstructionSize::Word)?;
                let reg = self.next_instruction(InstructionSize::Small)?;
                let reg = Register::try_from(reg)?;
                Ok(Instruction::JltReg(jump_to.into(), reg))
            }
            OpCode::Jmp => {
                let jump_to = self.next_instruction(InstructionSize::Word)?;
                Ok(Instruction::Jmp(jump_to.into()))
            }
            OpCode::Int => {
                let address = self.next_instruction(InstructionSize::Small)?;
                Ok(Instruction::Int(address))
            }
            OpCode::Rti => Ok(Instruction::Rti),
        }
    }

    fn execute(&mut self, instruction: Instruction) -> Result<ControlFlow> {
        match instruction {
            Instruction::MovLitReg(reg, val) => self.registers.set(reg, val),
            Instruction::MovRegReg(to, from) => {
                let val = self.registers.fetch(from);
                self.registers.set(to, val);
            }
            Instruction::MovRegMem(reg, address) => {
                let val = self.registers.fetch(reg);
                self.memory.write_word(address, val)?;
            }
            Instruction::MovLitMem(address, val) => {
                self.memory.write_word(address, val)?;
            }
            Instruction::MovMemReg(address, reg) => {
                let value = self.memory.read_word(address)?;
                self.registers.set(reg, value)
            }
            Instruction::MovRegPtrReg(address, from) => {
                let address = self.registers.fetch(address);
                let val = self.registers.fetch(from);
                self.memory.write_word(address, val)?;
            }
            Instruction::MovLitRegPtr(reg, lit) => {
                let address = self.registers.fetch(reg);
                self.memory.write_word(address, lit)?;
            }
            Instruction::Mov8LitReg(reg, lit) => self.registers.set(reg, lit as u16),
            Instruction::Mov8RegReg(from, to) => {
                let val = self.registers.fetch(from);
                let val = val & 0xFF;
                self.registers.set(to, val);
            }
            Instruction::Mov8RegMem(reg, address) => {
                let val = self.registers.fetch(reg);
                let val = val & 0xFF;
                self.memory.write(address, val as u8)?;
            }
            Instruction::Mov8LitMem(address, val) => {
                self.memory.write(address, val)?;
            }
            Instruction::Mov8MemReg(address, reg) => {
                let val = self.memory.read(address)?;
                self.registers.set(reg, val as u16);
            }

            Instruction::AddRegReg(r1, r2) => {
                let r1_value = self.registers.fetch(r1);
                let r2_value = self.registers.fetch(r2);
                self.registers.set(r1, r1_value.wrapping_add(r2_value));
            }
            Instruction::AddLitReg(reg, lit) => {
                let reg_value = self.registers.fetch(reg);
                self.registers.set(reg, reg_value.wrapping_add(lit));
            }
            Instruction::SubRegReg(r1, r2) => {
                let r1_value = self.registers.fetch(r1);
                let r2_value = self.registers.fetch(r2);
                self.registers.set(r1, r1_value.wrapping_sub(r2_value));
            }
            Instruction::SubLitReg(reg, lit) => {
                let reg_value = self.registers.fetch(reg);
                self.registers.set(reg, reg_value.wrapping_sub(lit));
            }
            Instruction::MulRegReg(r1, r2) => {
                let r1_value = self.registers.fetch(r1);
                let r2_value = self.registers.fetch(r2);
                self.registers.set(r1, r1_value.wrapping_mul(r2_value));
            }
            Instruction::MulLitReg(reg, lit) => {
                let reg_value = self.registers.fetch(reg);
                self.registers.set(reg, reg_value.wrapping_mul(lit));
            }
            Instruction::IncReg(reg) => {
                let reg_val = self.registers.fetch(reg);
                self.registers.set(reg, reg_val.wrapping_add(1));
            }
            Instruction::DecReg(reg) => {
                let reg_val = self.registers.fetch(reg);
                self.registers.set(reg, reg_val.wrapping_sub(1));
            }

            Instruction::LshLitReg(reg, lit) => {
                let reg_val = self.registers.fetch(reg);
                let val = reg_val << lit;
                self.registers.set(reg, val)
            }
            Instruction::LshRegReg(r1, r2) => {
                let r1_val = self.registers.fetch(r1);
                let r2_val = self.registers.fetch(r2);
                let val = r1_val << r2_val;
                self.registers.set(r1, val);
            }
            Instruction::RshLitReg(reg, lit) => {
                let reg_val = self.registers.fetch(reg);
                let val = reg_val >> lit;
                self.registers.set(reg, val)
            }
            Instruction::RshRegReg(r1, r2) => {
                let r1_val = self.registers.fetch(r1);
                let r2_val = self.registers.fetch(r2);
                let val = r1_val >> r2_val;
                self.registers.set(r1, val);
            }
            Instruction::AndLitReg(reg, lit) => {
                let reg_val = self.registers.fetch(reg);
                let val = reg_val & lit;
                self.registers.set(reg, val)
            }
            Instruction::AndRegReg(r1, r2) => {
                let r1_val = self.registers.fetch(r1);
                let r2_val = self.registers.fetch(r2);
                let val = r1_val & r2_val;
                self.registers.set(r1, val);
            }
            Instruction::OrLitReg(reg, lit) => {
                let reg_val = self.registers.fetch(reg);
                let val = reg_val | lit;
                self.registers.set(reg, val)
            }
            Instruction::OrRegReg(r1, r2) => {
                let r1_val = self.registers.fetch(r1);
                let r2_val = self.registers.fetch(r2);
                let val = r1_val | r2_val;
                self.registers.set(r1, val);
            }
            Instruction::XorLitReg(reg, lit) => {
                let reg_val = self.registers.fetch(reg);
                let val = reg_val ^ lit;
                self.registers.set(reg, val)
            }
            Instruction::XorRegReg(r1, r2) => {
                let r1_val = self.registers.fetch(r1);
                let r2_val = self.registers.fetch(r2);
                let val = r1_val ^ r2_val;
                self.registers.set(r1, val);
            }
            Instruction::Not(reg) => {
                let reg_val = self.registers.fetch(reg);
                let val = !reg_val;
                self.registers.set(reg, val)
            }

            Instruction::JeqLit(address, lit) => {
                let ret_val = self.registers.fetch(Register::Acc);
                if lit == ret_val {
                    let address = address + self.start_address;
                    self.registers.set(Register::IP, address.into())
                }
            }
            Instruction::JeqReg(address, reg) => {
                let ret_val = self.registers.fetch(Register::Acc);
                let reg_val = self.registers.fetch(reg);
                if reg_val == ret_val {
                    let address = address + self.start_address;
                    self.registers.set(Register::IP, address.into())
                }
            }
            Instruction::JgtLit(address, lit) => {
                let ret_val = self.registers.fetch(Register::Acc);
                if lit > ret_val {
                    let address = address + self.start_address;
                    self.registers.set(Register::IP, address.into())
                }
            }
            Instruction::JgtReg(address, reg) => {
                let ret_val = self.registers.fetch(Register::Acc);
                let reg_val = self.registers.fetch(reg);
                if reg_val > ret_val {
                    let address = address + self.start_address;
                    self.registers.set(Register::IP, address.into())
                }
            }
            Instruction::JneLit(address, lit) => {
                let ret_val = self.registers.fetch(Register::Acc);
                if lit != ret_val {
                    let address = address + self.start_address;
                    self.registers.set(Register::IP, address.into())
                }
            }
            Instruction::JneReg(address, reg) => {
                let ret_val = self.registers.fetch(Register::Acc);
                let reg_val = self.registers.fetch(reg);
                if reg_val != ret_val {
                    let address = address + self.start_address;
                    self.registers.set(Register::IP, address.into())
                }
            }
            Instruction::JgeLit(address, lit) => {
                let ret_val = self.registers.fetch(Register::Acc);
                if lit >= ret_val {
                    let address = address + self.start_address;
                    self.registers.set(Register::IP, address.into())
                }
            }
            Instruction::JgeReg(address, reg) => {
                let ret_val = self.registers.fetch(Register::Acc);
                let reg_val = self.registers.fetch(reg);
                if reg_val >= ret_val {
                    let address = address + self.start_address;
                    self.registers.set(Register::IP, address.into())
                }
            }
            Instruction::JleLit(address, lit) => {
                let ret_val = self.registers.fetch(Register::Acc);
                if lit <= ret_val {
                    let address = address + self.start_address;
                    self.registers.set(Register::IP, address.into());
                }
            }
            Instruction::JleReg(address, reg) => {
                let ret_val = self.registers.fetch(Register::Acc);
                let reg_val = self.registers.fetch(reg);
                if reg_val <= ret_val {
                    let address = address + self.start_address;
                    self.registers.set(Register::IP, address.into())
                }
            }
            Instruction::JltLit(address, lit) => {
                let ret_val = self.registers.fetch(Register::Acc);
                if lit < ret_val {
                    let address = address + self.start_address;
                    self.registers.set(Register::IP, address.into())
                }
            }
            Instruction::JltReg(address, reg) => {
                let ret_val = self.registers.fetch(Register::Acc);
                let reg_val = self.registers.fetch(reg);
                if reg_val < ret_val {
                    let address = address + self.start_address;
                    self.registers.set(Register::IP, address.into())
                }
            }
            Instruction::Jmp(address) => {
                let address = address + self.start_address;
                self.registers.set(Register::IP, address.into())
            }

            Instruction::PushLit(val) => self.push_stack(val)?,
            Instruction::PopReg(reg) => {
                let val = self.pop_stack()?;
                self.registers.set(reg, val);
            }
            Instruction::Call(address) => self.call_address(address)?,
            Instruction::CallRegPtr(reg) => {
                let address = self.registers.fetch(reg);
                self.call_address(address.into())?;
            }
            Instruction::Ret => self.restore_stack()?,
            Instruction::Halt(code) => return Ok(ControlFlow::Halt(code)),
            Instruction::Int(interrupt) => self.handle_interrupt(interrupt)?,
            Instruction::Rti => {
                self.in_interrupt = false;
                self.restore_stack()?;
            }
        }
        Ok(ControlFlow::Continue)
    }

    fn next_instruction(&mut self, size: InstructionSize) -> Result<u16> {
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

    fn call_address(&mut self, address: Word) -> Result<()> {
        self.save_stack()?;
        let address = address + self.start_address;
        self.registers.set(Register::IP, address.into());
        Ok(())
    }

    fn save_stack(&mut self) -> Result<()> {
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

        Ok(())
    }

    fn restore_stack(&mut self) -> Result<()> {
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

        let prev_frame_ptr = frame_ptr + frame_size.into();
        self.registers.set(Register::FP, prev_frame_ptr.into());

        Ok(())
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

    pub fn handle_interrupt(&mut self, idx: impl Into<u16>) -> Result<()> {
        let interrupt_idx = idx.into() & 0xF;

        // if the interrupt is unmasked (its 0) on the interrupt mask, then we should not enter the
        // interrupt handler
        let is_unmasked = (1 << interrupt_idx) & self.registers.fetch(Register::IM);
        if is_unmasked == 0 {
            return Ok(());
        }

        let handler_pointer = self.interrupt_table + (interrupt_idx * 2).into();
        let address = self.memory.read_word(handler_pointer)?;

        // if we are already within an interrupt (calling an interrupt from another), we don't save
        // the stack state
        if !self.in_interrupt {
            self.save_stack()?;
        }

        self.in_interrupt = true;
        self.registers.set(Register::IP, address);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Memory {
        memory: [u8; u16::MAX as usize],
    }

    impl Memory {
        pub fn new() -> Self {
            Self {
                memory: [0; u16::MAX as usize],
            }
        }
    }

    impl Addressable for Memory {
        fn read<W>(&self, address: W) -> crate::memory::Result<u8>
        where
            W: Into<Word> + Copy,
        {
            Ok(self.memory[usize::from(address.into())])
        }

        fn write<W>(&mut self, address: W, byte: impl Into<u8>) -> crate::memory::Result<()>
        where
            W: Into<Word> + Copy,
        {
            self.memory[usize::from(address.into())] = byte.into();
            Ok(())
        }
    }

    #[test]
    fn test_mov_lit_reg() {
        let mut memory = Memory::new();
        // mov r1, $ff
        memory.write(0x0000, OpCode::MovLitReg).unwrap();
        memory.write(0x0001, Register::R1).unwrap();
        memory.write_word(0x0002, 0x00FF).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.step().unwrap();
        assert_eq!(cpu.registers.fetch(Register::R1), 0xFF);
    }

    #[test]
    fn test_mov_reg_reg() {
        let mut memory = Memory::new();
        // mov r1, $ff
        memory.write(0x0000, OpCode::MovLitReg).unwrap();
        memory.write(0x0001, Register::R1).unwrap();
        memory.write_word(0x0002, 0x00FF).unwrap();

        // mov r2, r1
        memory.write(0x0004, OpCode::MovRegReg).unwrap();
        memory.write(0x0005, Register::R2).unwrap();
        memory.write(0x0006, Register::R1).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.step().unwrap();
        cpu.step().unwrap();
        assert_eq!(cpu.registers.fetch(Register::R2), 0xFF);
    }

    #[test]
    fn test_mov_reg_mem() {
        let mut memory = Memory::new();
        // mov r1, $ff
        memory.write(0x0000, OpCode::MovLitReg).unwrap();
        memory.write(0x0001, Register::R1).unwrap();
        memory.write_word(0x0002, 0x00FF).unwrap();

        // mov &[$0100], r1
        memory.write(0x0004, OpCode::MovRegMem).unwrap();
        memory.write_word(0x0005, 0x0100).unwrap();
        memory.write(0x0007, Register::R1).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.step().unwrap();
        cpu.step().unwrap();

        assert_eq!(cpu.memory.read(0x0100).unwrap(), 0xFF);
    }

    #[test]
    fn test_mov_mem_reg() {
        let mut memory = Memory::new();
        memory.write_word(0x0100, 0xabcd).unwrap();

        // mov r1, &[$0100]
        memory.write(0x0000, OpCode::MovMemReg).unwrap();
        memory.write(0x0001, Register::R1).unwrap();
        memory.write_word(0x0002, 0x0100).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::R1), 0xabcd);
    }

    #[test]
    fn test_jeq_reg() {
        let mut memory = Memory::new();

        // jeq &[$0100], r1
        memory.write(0x0000, OpCode::JeqReg).unwrap();
        memory.write_word(0x0001, 0x0100).unwrap();
        memory.write(0x0003, Register::R1).unwrap();

        // jeq &[$0100], r1
        memory.write(0x0004, OpCode::JeqReg).unwrap();
        memory.write_word(0x0005, 0x0100).unwrap();
        memory.write(0x0007, Register::R1).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.registers.set(Register::Acc, 0xabcd);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0004);

        cpu.registers.set(Register::R1, 0xabcd);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0100);
    }

    #[test]
    fn test_jeq_lit() {
        let mut memory = Memory::new();

        // jeq &[$0100], $1234
        memory.write(0x0000, OpCode::JeqLit).unwrap();
        memory.write_word(0x0001, 0x0100).unwrap();
        memory.write_word(0x0003, 0x1234).unwrap();

        // jeq &[$0100], $abcd
        memory.write(0x0005, OpCode::JeqLit).unwrap();
        memory.write_word(0x0006, 0x0100).unwrap();
        memory.write_word(0x0008, 0xabcd).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.registers.set(Register::Acc, 0xabcd);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0005);

        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0100);
    }

    #[test]
    fn test_jgt_reg() {
        let mut memory = Memory::new();

        // jgt &[$0100], r1
        memory.write(0x0000, OpCode::JgtReg).unwrap();
        memory.write_word(0x0001, 0x0100).unwrap();
        memory.write(0x0003, Register::R1).unwrap();

        // jgt &[$0100], r1
        memory.write(0x0004, OpCode::JgtReg).unwrap();
        memory.write_word(0x0005, 0x0100).unwrap();
        memory.write(0x0007, Register::R1).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.registers.set(Register::Acc, 0xabcd);
        cpu.registers.set(Register::R1, 0xabcc);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0004);

        cpu.registers.set(Register::R1, 0xffff);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0100);
    }

    #[test]
    fn test_jgt_lit() {
        let mut memory = Memory::new();

        // jgt &[$0100], $1234
        memory.write(0x0000, OpCode::JgtLit).unwrap();
        memory.write_word(0x0001, 0x0100).unwrap();
        memory.write_word(0x0003, 0x1234).unwrap();

        // jgt &[$0100], $abcd
        memory.write(0x0005, OpCode::JgtLit).unwrap();
        memory.write_word(0x0006, 0x0100).unwrap();
        memory.write_word(0x0008, 0xffff).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.registers.set(Register::Acc, 0xabcd);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0005);

        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0100);
    }

    #[test]
    fn test_jne_reg() {
        let mut memory = Memory::new();

        // jne &[$0100], r1
        memory.write(0x0000, OpCode::JneReg).unwrap();
        memory.write_word(0x0001, 0x0100).unwrap();
        memory.write(0x0003, Register::R1).unwrap();

        // jne &[$0100], r1
        memory.write(0x0004, OpCode::JneReg).unwrap();
        memory.write_word(0x0005, 0x0100).unwrap();
        memory.write(0x0007, Register::R1).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.registers.set(Register::Acc, 0xabcd);
        cpu.registers.set(Register::R1, 0xabcd);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0004);

        cpu.registers.set(Register::R1, 0xffff);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0100);
    }

    #[test]
    fn test_jne_lit() {
        let mut memory = Memory::new();

        // jne &[$0100], $abcd
        memory.write(0x0000, OpCode::JneLit).unwrap();
        memory.write_word(0x0001, 0x0100).unwrap();
        memory.write_word(0x0003, 0xabcd).unwrap();

        // jne &[$0100], $abce
        memory.write(0x0005, OpCode::JneLit).unwrap();
        memory.write_word(0x0006, 0x0100).unwrap();
        memory.write_word(0x0008, 0xffff).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.registers.set(Register::Acc, 0xabcd);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0005);

        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0100);
    }

    #[test]
    fn test_jge_reg() {
        let mut memory = Memory::new();

        // jge &[$0100], r1
        memory.write(0x0000, OpCode::JgeReg).unwrap();
        memory.write_word(0x0001, 0x0100).unwrap();
        memory.write(0x0003, Register::R1).unwrap();

        // jge &[$0100], r1
        memory.write(0x0004, OpCode::JgeReg).unwrap();
        memory.write_word(0x0005, 0x0100).unwrap();
        memory.write(0x0007, Register::R1).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.registers.set(Register::Acc, 0xabcd);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0004);

        cpu.registers.set(Register::R1, 0xabcd);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0100);
    }

    #[test]
    fn test_jge_lit() {
        let mut memory = Memory::new();

        // jge &[$0100], $0000
        memory.write(0x0000, OpCode::JgeLit).unwrap();
        memory.write_word(0x0001, 0x0100).unwrap();
        memory.write_word(0x0003, 0x0000).unwrap();

        // jge &[$0100], $abcd
        memory.write(0x0005, OpCode::JgeLit).unwrap();
        memory.write_word(0x0006, 0x0100).unwrap();
        memory.write_word(0x0008, 0xffff).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.registers.set(Register::Acc, 0xabcd);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0005);

        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0100);
    }

    #[test]
    fn test_jle_reg() {
        let mut memory = Memory::new();

        // jle &[$0100], r1
        memory.write(0x0000, OpCode::JleReg).unwrap();
        memory.write_word(0x0001, 0x0100).unwrap();
        memory.write(0x0003, Register::R1).unwrap();

        // jle &[$0100], r1
        memory.write(0x0004, OpCode::JleReg).unwrap();
        memory.write_word(0x0005, 0x0100).unwrap();
        memory.write(0x0007, Register::R1).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.registers.set(Register::Acc, 0xabcd);
        cpu.registers.set(Register::R1, 0xffff);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0004);

        cpu.registers.set(Register::R1, 0xabcd);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0100);
    }

    #[test]
    fn test_jle_lit() {
        let mut memory = Memory::new();

        // jle &[$0100], $ffff
        memory.write(0x0000, OpCode::JleLit).unwrap();
        memory.write_word(0x0001, 0x0100).unwrap();
        memory.write_word(0x0003, 0xffff).unwrap();

        // jle &[$0100], $abcd
        memory.write(0x0005, OpCode::JleLit).unwrap();
        memory.write_word(0x0006, 0x0100).unwrap();
        memory.write_word(0x0008, 0xabcd).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.registers.set(Register::Acc, 0xabcd);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0005);

        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0100);
    }

    #[test]
    fn test_jlt_reg() {
        let mut memory = Memory::new();

        // jlt &[$0100], r1
        memory.write(0x0000, OpCode::JltReg).unwrap();
        memory.write_word(0x0001, 0x0100).unwrap();
        memory.write(0x0003, Register::R1).unwrap();

        // jlt &[$0100], r1
        memory.write(0x0004, OpCode::JltReg).unwrap();
        memory.write_word(0x0005, 0x0100).unwrap();
        memory.write(0x0007, Register::R1).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.registers.set(Register::Acc, 0xabcd);
        cpu.registers.set(Register::R1, 0xabcd);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0004);

        cpu.registers.set(Register::R1, 0xabcc);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0100);
    }

    #[test]
    fn test_jlt_lit() {
        let mut memory = Memory::new();

        // jlt &[$0100], $abcd
        memory.write(0x0000, OpCode::JltLit).unwrap();
        memory.write_word(0x0001, 0x0100).unwrap();
        memory.write_word(0x0003, 0xabcd).unwrap();

        // jlt &[$0100], $abcc
        memory.write(0x0005, OpCode::JltLit).unwrap();
        memory.write_word(0x0006, 0x0100).unwrap();
        memory.write_word(0x0008, 0xabcc).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.registers.set(Register::Acc, 0xabcd);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0005);

        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0100);
    }

    #[test]
    fn test_jmp_lit() {
        let mut memory = Memory::new();

        // jmp &[$0100]
        memory.write(0x0000, OpCode::Jmp).unwrap();
        memory.write_word(0x0001, 0x0100).unwrap();

        let mut cpu = Cpu::new(memory, 0, 0x8000, 0x1000);
        cpu.step().unwrap();

        assert_eq!(cpu.registers.fetch(Register::IP), 0x0100);
    }
}
