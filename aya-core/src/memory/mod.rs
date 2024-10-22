mod error;
mod linear_memory;
mod memory_mapper;
mod output_memory;

pub use error::{Error, Result};
pub use linear_memory::LinearMemory;
pub use memory_mapper::{MappingMode, MemoryMapper};
pub use output_memory::OutputMemory;

use crate::word::Word;

pub trait Addressable<const SIZE: usize> {
    fn read(&self, address: Word<SIZE>) -> Result<SIZE, u8>;
    fn write(&mut self, address: Word<SIZE>, byte: u8) -> Result<SIZE, ()>;

    fn read_word(&self, address: Word<SIZE>) -> Result<SIZE, u16> {
        let first = self.read(address)?;
        let second = self.read(address.next()?)?;
        Ok(u16::from_le_bytes([first, second]))
    }

    fn write_word(&mut self, address: Word<SIZE>, word: u16) -> Result<SIZE, ()> {
        let [lower, upper] = word.to_le_bytes();
        self.write(address, lower)?;
        self.write(address.next()?, upper)?;
        Ok(())
    }

    fn inspect_address<W: TryInto<Word<SIZE>>>(&self, address: W, size: usize) -> Result<SIZE, Vec<u16>> {
        let mut curr = match address.try_into() {
            Ok(curr) => curr,
            Err(_) => unreachable!(),
        };

        let mut mem = Vec::with_capacity(size);

        for _ in 0..size {
            mem.push(self.read_word(curr)?);
            let Ok(next) = curr.next_word() else {
                break;
            };
            curr = next;
        }

        Ok(mem)
    }
}
