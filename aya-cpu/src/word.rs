use std::{fmt, ops};

use crate::memory::Error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word(u16);

impl fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl Word {
    pub fn next(&self) -> Result<Word> {
        let Some(next) = self.0.checked_add(1) else { return Err(Error::StackOverflow) };
        Ok(Word(next))
    }

    pub fn next_word(&self) -> Result<Word> {
        let Some(next) = self.0.checked_add(2) else { return Err(Error::StackOverflow) };
        Ok(Word(next))
    }

    pub fn prev(&self) -> Result<Word> {
        let Some(prev) = self.0.checked_sub(1) else { return Err(Error::StackUnderflow) };
        Ok(Word(prev))
    }

    pub fn prev_word(&self) -> Result<Word> {
        let Some(prev) = self.0.checked_sub(2) else { return Err(Error::StackUnderflow) };
        Ok(Word(prev))
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        Word(value)
    }
}

impl From<Word> for usize {
    fn from(word: Word) -> Self {
        word.0 as usize
    }
}

impl From<Word> for u16 {
    fn from(word: Word) -> Self {
        word.0
    }
}

impl ops::Sub for Word {
    type Output = Word;

    fn sub(self, rhs: Self) -> Self::Output {
        Word(self.0 - rhs.0)
    }
}

impl ops::Add for Word {
    type Output = Word;

    fn add(self, rhs: Self) -> Self::Output {
        Word(self.0 + rhs.0)
    }
}
