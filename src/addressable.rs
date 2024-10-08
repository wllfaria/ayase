use crate::Word;

pub trait Addressable {
    fn read(&self, address: Word) -> Result<u8, String>;
    fn write(&mut self, address: Word, byte: u8) -> Result<(), String>;

    fn read_word(&self, address: Word) -> Result<u16, String> {
        let first = self.read(address)? as u16;
        let second = self.read(address.next())? as u16;
        Ok(first | (second << 8))
    }

    fn write_word(&mut self, address: Word, word: u16) -> Result<(), String> {
        let lower = (word & 0xff) as u8;
        let upper = ((word & 0xff00) >> 8) as u8;
        self.write(address, lower)?;
        self.write(address.next(), upper)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct LinearMemory<const SIZE: usize> {
    inner: [u8; SIZE],
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

impl<const SIZE: usize> Addressable for LinearMemory<SIZE> {
    fn read(&self, address: Word) -> Result<u8, String> {
        match self.inner.get::<usize>(address.into()) {
            Some(byte) => Ok(*byte),
            None => Err("someting".into()),
        }
    }

    fn write(&mut self, address: Word, byte: u8) -> Result<(), String> {
        self.inner[usize::from(address)] = byte;
        Ok(())
    }
}
