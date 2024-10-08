mod addressable;

use addressable::{Addressable, LinearMemory};
use std::fmt;

const MEMORY_SIZE: usize = u16::MAX as usize;

#[derive(Debug, Copy, Clone)]
pub struct Word(u16);

impl fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::UpperHex for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl Word {
    pub fn next(&self) -> Word {
        Word(self.0 + 1)
    }

    pub fn next_word(&self) -> Word {
        Word(self.0 + 2)
    }
}

impl From<Word> for usize {
    fn from(word: Word) -> Self {
        word.0 as usize
    }
}

impl From<Word> for u16 {
    fn from(word: Word) -> Self {
        word.0
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum Register {
    Ret,
    IP,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    SP,
    FP,
}

impl Register {
    pub const fn len() -> usize {
        12
    }

    pub const fn is_empty() -> bool {
        Register::len() == 0
    }
}

impl From<Register> for usize {
    fn from(register: Register) -> Self {
        register as usize
    }
}

impl From<Register> for u16 {
    fn from(val: Register) -> Self {
        val as u16
    }
}

#[derive(Debug, Default)]
pub struct Registers {
    inner: [u16; Register::len()],
}

impl Registers {
    pub fn new() -> Self {
        Self {
            inner: [0; Register::len()],
        }
    }

    pub fn fetch(&self, register: Register) -> Word {
        Word(self.inner[register as usize])
    }

    pub fn set(&mut self, register: Register, value: Word) {
        self.inner[register as usize] = value.into();
    }
}

#[derive(Debug)]
pub struct Cpu<A: Addressable> {
    registers: Registers,
    memory: A,
}

impl<A: Addressable> Cpu<A> {
    pub fn new(memory: A) -> Self {
        Self {
            registers: Registers::default(),
            memory,
        }
    }

    pub fn step(&mut self) -> Result<(), String> {
        let ip = self.registers.fetch(Register::IP);
        let instruction = self.memory.read_word(ip)?;
        self.registers.set(Register::IP, ip.next_word());
        println!("0x{ip:04X} @ 0x{instruction:02x}");
        Ok(())
    }
}

fn main() {
    let memory = LinearMemory::<MEMORY_SIZE>::default();
    let mut cpu = Cpu::new(memory);
    cpu.step().ok();
}
