use super::Result;
use crate::word::Word;

pub trait Addressable {
    fn read<W>(&self, address: W) -> Result<u8>
    where
        W: Into<Word> + Copy;

    fn write<W>(&mut self, address: W, byte: impl Into<u8>) -> Result<()>
    where
        W: Into<Word> + Copy;

    fn read_word<W>(&self, address: W) -> Result<u16>
    where
        W: Into<Word> + Copy,
    {
        let first = self.read(address)?;
        let address: Word = address.into();
        let second = self.read(address.next()?)?;
        Ok(u16::from_le_bytes([first, second]))
    }

    fn write_word<W>(&mut self, address: W, word: u16) -> Result<()>
    where
        W: Into<Word> + Copy,
    {
        let [lower, upper] = word.to_le_bytes();
        self.write(address, lower)?;
        let address = address.into();
        self.write(address.next()?, upper)?;
        Ok(())
    }

    fn inspect_address<W>(&self, address: W, size: usize) -> Result<Vec<u16>>
    where
        W: TryInto<Word>,
    {
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
