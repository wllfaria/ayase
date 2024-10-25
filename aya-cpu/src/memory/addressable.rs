use super::Result;
use crate::word::Word;

pub trait Addressable {
    fn read(&self, address: Word) -> Result<u8>;
    fn write(&mut self, address: Word, byte: u8) -> Result<()>;

    fn read_word(&self, address: Word) -> Result<u16> {
        let first = self.read(address)?;
        let second = self.read(address.next()?)?;
        Ok(u16::from_le_bytes([first, second]))
    }

    fn write_word(&mut self, address: Word, word: u16) -> Result<()> {
        let [lower, upper] = word.to_le_bytes();
        self.write(address, lower)?;
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
