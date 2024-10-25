use aya_cpu::memory::{Addressable, Error, Result};
use aya_cpu::word::Word;

use super::memory_mapper::KB4;

pub struct SpriteMemory {
    inner: [u8; KB4],
}

impl std::fmt::Debug for SpriteMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl SpriteMemory {
    pub const fn new() -> Self {
        Self { inner: [0; KB4] }
    }
}

impl Default for SpriteMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl Addressable for SpriteMemory {
    fn read(&self, address: Word) -> Result<u8> {
        match self.inner.get::<usize>(address.into()) {
            Some(byte) => Ok(*byte),
            None => Err(Error::InvalidAddress(address.into())),
        }
    }

    fn write(&mut self, address: Word, byte: u8) -> Result<()> {
        self.inner[usize::from(address)] = byte;
        Ok(())
    }
}
