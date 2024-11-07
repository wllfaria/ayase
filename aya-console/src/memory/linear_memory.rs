use aya_cpu::memory::{Addressable, Result};
use aya_cpu::word::Word;

#[derive(Debug)]
pub struct LinearMemory<const SIZE: usize> {
    memory: [u8; SIZE],
}

impl<const SIZE: usize> Default for LinearMemory<SIZE> {
    fn default() -> Self {
        Self { memory: [0; SIZE] }
    }
}

impl<const SIZE: usize> Addressable for LinearMemory<SIZE> {
    fn read<W>(&self, address: W) -> Result<u8>
    where
        W: Into<Word> + Copy,
    {
        let address = address.into();
        Ok(self.memory[usize::from(address)])
    }

    fn write<W>(&mut self, address: W, byte: impl Into<u8>) -> Result<()>
    where
        W: Into<Word> + Copy,
    {
        self.memory[usize::from(address.into())] = byte.into();
        Ok(())
    }
}

impl<const SIZE: usize> From<&[u8]> for LinearMemory<SIZE> {
    fn from(value: &[u8]) -> Self {
        let mut memory = [0; SIZE];
        memory[..value.len()].copy_from_slice(&value[..value.len()]);
        Self { memory }
    }
}
