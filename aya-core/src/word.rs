use std::fmt;

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
    pub fn next(&self) -> Word<SIZE> {
        Word(self.0 + 1)
    }

    pub fn next_word(&self) -> Word<SIZE> {
        Word(self.0 + 2)
    }
}

impl<const SIZE: usize> From<u16> for Word<SIZE> {
    fn from(value: u16) -> Self {
        Word(value)
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
