use aya_cpu::memory::{Addressable, Result};
use aya_cpu::word::Word;

use super::BG_MEMORY;

#[derive(Debug)]
pub struct BackgroundMemory {
    memory: [u8; BG_MEMORY],
}

impl Default for BackgroundMemory {
    fn default() -> Self {
        Self { memory: [0; BG_MEMORY] }
    }
}

impl Addressable for BackgroundMemory {
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
