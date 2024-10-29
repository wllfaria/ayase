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
    fn read<W>(&self, address: W) -> Result<u8>
    where
        W: Into<Word> + Copy,
    {
        Ok(self.memory[usize::from(address.into())])
    }

    fn write<W>(&mut self, address: W, byte: u8) -> Result<()>
    where
        W: Into<Word> + Copy,
    {
        self.memory[usize::from(address.into())] = byte;
        Ok(())
    }
}
