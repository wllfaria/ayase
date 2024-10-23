use aya_core::memory::{Addressable, Result};
use aya_core::word::Word;

#[derive(Debug)]
pub struct OutputDevice<const SIZE: usize> {
    memory: [u8; SIZE],
}

impl<const SIZE: usize> Default for OutputDevice<SIZE> {
    fn default() -> Self {
        Self { memory: [0; SIZE] }
    }
}

impl<const SIZE: usize> Addressable<SIZE> for OutputDevice<SIZE> {
    fn read(&self, address: Word<SIZE>) -> Result<SIZE, u8> {
        Ok(self.memory[usize::from(address)])
    }

    fn write(&mut self, address: Word<SIZE>, byte: u8) -> Result<SIZE, ()> {
        self.memory[usize::from(address)] = byte;
        Ok(())
    }

    fn write_word(&mut self, _: Word<SIZE>, _: u16) -> Result<SIZE, ()> {
        Ok(())
    }

    fn read_word(&self, _: Word<SIZE>) -> Result<SIZE, u16> {
        Ok(0)
    }
}
