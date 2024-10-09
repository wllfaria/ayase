use std::fmt;

use crate::memory::Error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Copy, Clone)]
pub struct Word<const SIZE: usize>(u16);

impl<const SIZE: usize> fmt::Display for Word<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl<const SIZE: usize> fmt::UpperHex for Word<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl<const SIZE: usize> Word<SIZE> {
    pub fn next(&self) -> Result<Word<SIZE>> {
        let Some(next) = self.0.checked_add(1) else { return Err(Error::StackOverflow) };
        if next as usize >= SIZE {
            return Err(Error::StackOverflow);
        };
        Ok(Word(next))
    }

    pub fn next_word(&self) -> Result<Word<SIZE>> {
        let Some(next) = self.0.checked_add(2) else { return Err(Error::StackOverflow) };
        if next as usize >= SIZE {
            return Err(Error::StackOverflow);
        };
        Ok(Word(next))
    }

    pub fn prev(&self) -> Result<Word<SIZE>> {
        let Some(prev) = self.0.checked_sub(1) else { return Err(Error::StackUnderflow) };
        Ok(Word(prev))
    }

    pub fn prev_word(&self) -> Result<Word<SIZE>> {
        let Some(prev) = self.0.checked_sub(2) else { return Err(Error::StackUnderflow) };
        Ok(Word(prev))
    }
}

impl<const SIZE: usize> TryFrom<u16> for Word<SIZE> {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self> {
        if (value as usize) < SIZE {
            Ok(Word(value))
        } else {
            Err(Error::InvalidAddress(format!(
                "address 0x{value:04X?} is out of memory bounds"
            )))
        }
    }
}

impl<const SIZE: usize> From<Word<SIZE>> for usize {
    fn from(word: Word<SIZE>) -> Self {
        word.0 as usize
    }
}

impl<const SIZE: usize> From<Word<SIZE>> for u16 {
    fn from(word: Word<SIZE>) -> Self {
        word.0
    }
}
