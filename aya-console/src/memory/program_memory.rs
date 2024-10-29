use aya_cpu::memory::{Addressable, Result};
use aya_cpu::word::Word;

use super::KB16;

#[derive(Debug)]
pub struct ProgramMemory {
    memory: [u8; KB16],
}

impl Default for ProgramMemory {
    fn default() -> Self {
        Self { memory: [0; KB16] }
    }
}

impl Addressable for ProgramMemory {
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
