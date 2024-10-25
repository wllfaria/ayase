use aya_cpu::memory::{Addressable, Result};
use aya_cpu::word::Word;

#[derive(Debug)]
pub struct ProgramMemory {}

impl Addressable for ProgramMemory {
    fn read(&self, address: Word) -> Result<u8> {}

    fn write(&self, address: Word, byte: u8) -> Result<u8> {}
}
