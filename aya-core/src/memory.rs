use crate::word::Word;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidAddress(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

pub trait Addressable<const SIZE: usize> {
    fn read(&self, address: Word<SIZE>) -> Result<u8>;
    fn write(&mut self, address: Word<SIZE>, byte: u8) -> Result<()>;

    fn read_word(&self, address: Word<SIZE>) -> Result<u16> {
        let first = self.read(address)? as u16;
        let second = self.read(address.next())? as u16;
        Ok(first | (second << 8))
    }

    fn write_word(&mut self, address: Word<SIZE>, word: u16) -> Result<()> {
        let lower = (word & 0xff) as u8;
        let upper = ((word & 0xff00) >> 8) as u8;
        self.write(address, lower)?;
        self.write(address.next(), upper)?;
        Ok(())
    }

    #[cfg(debug_assertions)]
    fn inspect_address(&self, address: Word<SIZE>, size: usize) -> Result<()> {
        let mut curr = address;
        print!("0x{address:04X}: ");
        for _ in 0..size {
            print!("0x{:02X} ", self.read(curr)?);
            curr = curr.next();
        }
        println!();
        Ok(())
    }
}

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
    fn read(&self, address: Word<SIZE>) -> Result<u8> {
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
