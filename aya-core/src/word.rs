use std::{fmt, ops};

use crate::memory::Error;

type Result<const MEM_SIZE: usize, T> = std::result::Result<T, Error<MEM_SIZE>>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
    pub fn next(&self) -> Result<SIZE, Word<SIZE>> {
        let Some(next) = self.0.checked_add(1) else { return Err(Error::StackOverflow) };
        if next as usize >= SIZE {
            return Err(Error::StackOverflow);
        };
        Ok(Word(next))
    }

    pub fn next_word(&self) -> Result<SIZE, Word<SIZE>> {
        let Some(next) = self.0.checked_add(2) else { return Err(Error::StackOverflow) };
        if next as usize >= SIZE {
            return Err(Error::StackOverflow);
        };
        Ok(Word(next))
    }

    pub fn prev(&self) -> Result<SIZE, Word<SIZE>> {
        let Some(prev) = self.0.checked_sub(1) else { return Err(Error::StackUnderflow) };
        Ok(Word(prev))
    }

    pub fn prev_word(&self) -> Result<SIZE, Word<SIZE>> {
        let Some(prev) = self.0.checked_sub(2) else { return Err(Error::StackUnderflow) };
        Ok(Word(prev))
    }
}

impl<const SIZE: usize> TryFrom<u16> for Word<SIZE> {
    type Error = Error<SIZE>;

    fn try_from(value: u16) -> Result<SIZE, Self> {
        if (value as usize) <= SIZE {
            Ok(Word(value))
        } else {
            Err(Error::InvalidAddress(value))
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

impl<const SIZE: usize> ops::Sub for Word<SIZE> {
    type Output = Word<SIZE>;

    fn sub(self, rhs: Self) -> Self::Output {
        Word(self.0 - rhs.0)
    }
}

impl<const SIZE: usize> ops::Add for Word<SIZE> {
    type Output = Word<SIZE>;

    fn add(self, rhs: Self) -> Self::Output {
        Word(self.0 + rhs.0)
    }
}
