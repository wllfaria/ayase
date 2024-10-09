use std::collections::VecDeque;

use super::{Addressable, Error, LinearMemory, OutputMemory, Result};
use crate::word::Word;

#[derive(Debug)]
pub enum MappableDevice<const SIZE: usize> {
    LinearMem(LinearMemory<SIZE>),
    Output(OutputMemory<SIZE>),
}

impl<const SIZE: usize> Addressable<SIZE> for MappableDevice<SIZE> {
    fn write(&mut self, address: Word<SIZE>, byte: u8) -> Result<SIZE, ()> {
        match self {
            MappableDevice::LinearMem(mem) => mem.write(address, byte),
            MappableDevice::Output(mem) => mem.write(address, byte),
        }
    }

    fn read(&mut self, address: Word<SIZE>) -> Result<SIZE, u8> {
        match self {
            MappableDevice::LinearMem(mem) => mem.read(address),
            MappableDevice::Output(mem) => mem.read(address),
        }
    }

    fn write_word(&mut self, address: Word<SIZE>, word: u16) -> Result<SIZE, ()> {
        match self {
            MappableDevice::LinearMem(mem) => mem.write_word(address, word),
            MappableDevice::Output(mem) => mem.write_word(address, word),
        }
    }

    fn read_word(&mut self, address: Word<SIZE>) -> Result<SIZE, u16> {
        match self {
            MappableDevice::LinearMem(mem) => mem.read_word(address),
            MappableDevice::Output(mem) => mem.read_word(address),
        }
    }
}

impl<const SIZE: usize> From<LinearMemory<SIZE>> for MappableDevice<SIZE> {
    fn from(memory: LinearMemory<SIZE>) -> Self {
        MappableDevice::LinearMem(memory)
    }
}

impl<const SIZE: usize> From<OutputMemory<SIZE>> for MappableDevice<SIZE> {
    fn from(memory: OutputMemory<SIZE>) -> Self {
        MappableDevice::Output(memory)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum MappingMode {
    Direct,
    Remap,
}

#[derive(Debug)]
struct MappedRegion<const SIZE: usize> {
    device: MappableDevice<SIZE>,
    start: Word<SIZE>,
    end: Word<SIZE>,
    mapping_mode: MappingMode,
}

#[derive(Debug, Default)]
pub struct MemoryMapper<const SIZE: usize> {
    regions: VecDeque<MappedRegion<SIZE>>,
}

impl<const SIZE: usize> MemoryMapper<SIZE> {
    pub fn map<W, D>(&mut self, device: D, start: W, end: W, mapping_mode: MappingMode) -> Result<SIZE, ()>
    where
        W: TryInto<Word<SIZE>, Error = Error<SIZE>>,
        D: Into<MappableDevice<SIZE>>,
    {
        let start = start.try_into()?;
        let end = end.try_into()?;
        self.regions.push_front(MappedRegion {
            device: device.into(),
            start,
            end,
            mapping_mode,
        });
        Ok(())
    }

    fn find_region(&mut self, address: Word<SIZE>) -> Option<&mut MappedRegion<SIZE>> {
        self.regions
            .iter_mut()
            .find(|region| address >= region.start && address <= region.end)
    }
}

impl<const SIZE: usize> Addressable<SIZE> for MemoryMapper<SIZE> {
    fn read(&mut self, address: Word<SIZE>) -> Result<SIZE, u8> {
        let Some(region) = self.find_region(address) else {
            return Err(Error::UnmappedAddress(address));
        };
        let address = match region.mapping_mode {
            MappingMode::Remap => address - region.start,
            MappingMode::Direct => address,
        };
        region.device.read(address)
    }

    fn write(&mut self, address: Word<SIZE>, byte: u8) -> Result<SIZE, ()> {
        let Some(region) = self.find_region(address) else {
            return Err(Error::UnmappedAddress(address));
        };
        let address = match region.mapping_mode {
            MappingMode::Remap => address - region.start,
            MappingMode::Direct => address,
        };
        region.device.write(address, byte)
    }

    fn read_word(&mut self, address: Word<SIZE>) -> Result<SIZE, u16> {
        let Some(region) = self.find_region(address) else {
            return Err(Error::UnmappedAddress(address));
        };
        let address = match region.mapping_mode {
            MappingMode::Remap => address - region.start,
            MappingMode::Direct => address,
        };
        region.device.read_word(address)
    }

    fn write_word(&mut self, address: Word<SIZE>, byte: u16) -> Result<SIZE, ()> {
        let Some(region) = self.find_region(address) else {
            return Err(Error::UnmappedAddress(address));
        };
        let address = match region.mapping_mode {
            MappingMode::Remap => address - region.start,
            MappingMode::Direct => address,
        };
        region.device.write_word(address, byte)
    }
}
