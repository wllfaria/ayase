use aya_cpu::memory::{Addressable, Error, Result};
use aya_cpu::word::Word;

pub struct DebugMemory {
    inner: [u8; u16::MAX as usize],
}

impl std::fmt::Debug for DebugMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl DebugMemory {
    pub fn new() -> Self {
        Self {
            inner: [0; u16::MAX as usize],
        }
    }
}

impl Default for DebugMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl Addressable for DebugMemory {
    fn read<W>(&self, address: W) -> Result<u8>
    where
        W: Into<Word> + Copy,
    {
        let address = address.into();
        match self.inner.get::<usize>(address.into()) {
            Some(byte) => Ok(*byte),
            None => Err(Error::InvalidAddress(address.into())),
        }
    }

    fn write<W>(&mut self, address: W, byte: u8) -> Result<()>
    where
        W: Into<Word> + Copy,
    {
        self.inner[usize::from(address.into())] = byte;
        Ok(())
    }
}
