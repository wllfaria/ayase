use aya_cpu::memory::{Addressable, Error, Result};
use aya_cpu::word::Word;

use super::TILE_MEMORY;

pub struct TileMemory {
    inner: [u8; TILE_MEMORY],
}

impl std::fmt::Debug for TileMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl TileMemory {
    pub fn new(sprites: &[u8]) -> Self {
        let mut inner = [0; TILE_MEMORY];
        inner[..sprites.len()].copy_from_slice(&sprites[..sprites.len()]);
        Self { inner }
    }
}

impl Addressable for TileMemory {
    fn read<W>(&self, address: W) -> Result<u8>
    where
        W: Into<Word> + Copy,
    {
        let address = address.into();
        match self.inner.get::<usize>(address.into()) {
            Some(byte) => Ok(*byte),
            None => Err(Error::InvalidAddress(address.into())),
        }
    }

    fn write<W>(&mut self, address: W, byte: u8) -> Result<()>
    where
        W: Into<Word> + Copy,
    {
        self.inner[usize::from(address.into())] = byte;
        Ok(())
    }
}
