use aya_cpu::memory::{Addressable, Result};
use aya_cpu::word::Word;

use super::KB16;

#[derive(Debug)]
pub struct StackMemory {
    memory: [u8; KB16],
}

impl Default for StackMemory {
    fn default() -> Self {
        Self { memory: [0; KB16] }
    }
}

impl Addressable for StackMemory {
    fn read(&self, address: Word) -> Result<u8> {
        Ok(self.memory[usize::from(address)])
    }

    fn write(&mut self, address: Word, byte: u8) -> Result<()> {
        self.memory[usize::from(address)] = byte;
        Ok(())
    }
}
