use aya_cpu::memory::{Addressable, Result};
use aya_cpu::word::Word;

use super::SPRITE_MEMORY;

#[derive(Debug)]
pub struct SpriteMemory {
    memory: [u8; SPRITE_MEMORY],
}

impl SpriteMemory {
    pub fn new() -> Self {
        Self {
            memory: [0; SPRITE_MEMORY],
        }
    }
}

impl Default for SpriteMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl Addressable for SpriteMemory {
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
