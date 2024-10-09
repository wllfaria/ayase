use super::{Addressable, Error, Result};
use crate::word::Word;

pub struct LinearMemory<const SIZE: usize> {
    inner: [u8; SIZE],
}

impl<const SIZE: usize> std::fmt::Debug for LinearMemory<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<const SIZE: usize> LinearMemory<SIZE> {
    pub const fn new() -> Self {
        Self { inner: [0; SIZE] }
    }
}

impl<const SIZE: usize> Default for LinearMemory<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize> Addressable<SIZE> for LinearMemory<SIZE> {
    fn read(&mut self, address: Word<SIZE>) -> Result<u8> {
        match self.inner.get::<usize>(address.into()) {
            Some(byte) => Ok(*byte),
            None => Err(Error::InvalidAddress(format!(
                "address 0x{address:04X?} is out of memory bounds"
            ))),
        }
    }

    fn write(&mut self, address: Word<SIZE>, byte: u8) -> Result<()> {
        self.inner[usize::from(address)] = byte;
        Ok(())
    }
}
