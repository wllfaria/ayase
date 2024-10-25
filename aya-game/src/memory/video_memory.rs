use aya_cpu::memory::{Addressable, Result};
use aya_cpu::word::Word;

use super::memory_mapper::KB8;

#[derive(Debug)]
pub struct VideoMemory {
    memory: [u8; KB8],
}

impl Default for VideoMemory {
    fn default() -> Self {
        Self { memory: [0; KB8] }
    }
}

impl Addressable for VideoMemory {
    fn read(&self, address: Word) -> Result<u8> {
        Ok(self.memory[usize::from(address)])
    }

    fn write(&mut self, address: Word, byte: u8) -> Result<()> {
        self.memory[usize::from(address)] = byte;
        Ok(())
    }

    fn write_word(&mut self, _: Word, _: u16) -> Result<()> {
        Ok(())
    }

    fn read_word(&self, _: Word) -> Result<u16> {
        Ok(0)
    }
}
