use std::collections::VecDeque;

use aya_cpu::memory::{Addressable, Error, Result};
use aya_cpu::word::Word;

use super::{ProgramMemory, SpriteMemory, StackMemory, VideoMemory};

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Devices {
    Program(ProgramMemory),
    Video(VideoMemory),
    Sprite(SpriteMemory),
    Stack(StackMemory),
}

impl Addressable for Devices {
    fn write(&mut self, address: Word, byte: u8) -> Result<()> {
        match self {
            Devices::Program(mem) => mem.write(address, byte),
            Devices::Video(mem) => mem.write(address, byte),
            Devices::Sprite(mem) => mem.write(address, byte),
            Devices::Stack(mem) => mem.write(address, byte),
        }
    }

    fn read(&self, address: Word) -> Result<u8> {
        match self {
            Devices::Program(mem) => mem.read(address),
            Devices::Video(mem) => mem.read(address),
            Devices::Sprite(mem) => mem.read(address),
            Devices::Stack(mem) => mem.read(address),
        }
    }

    fn write_word(&mut self, address: Word, word: u16) -> Result<()> {
        match self {
            Devices::Program(mem) => mem.write_word(address, word),
            Devices::Video(mem) => mem.write_word(address, word),
            Devices::Sprite(mem) => mem.write_word(address, word),
            Devices::Stack(mem) => mem.write_word(address, word),
        }
    }

    fn read_word(&self, address: Word) -> Result<u16> {
        match self {
            Devices::Program(mem) => mem.read_word(address),
            Devices::Video(mem) => mem.read_word(address),
            Devices::Sprite(mem) => mem.read_word(address),
            Devices::Stack(mem) => mem.read_word(address),
        }
    }
}

impl From<SpriteMemory> for Devices {
    fn from(mem: SpriteMemory) -> Self {
        Self::Sprite(mem)
    }
}

impl From<VideoMemory> for Devices {
    fn from(mem: VideoMemory) -> Self {
        Self::Video(mem)
    }
}

impl From<ProgramMemory> for Devices {
    fn from(mem: ProgramMemory) -> Self {
        Self::Program(mem)
    }
}

impl From<StackMemory> for Devices {
    fn from(mem: StackMemory) -> Self {
        Self::Stack(mem)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum MappingMode {
    Direct,
    Remap,
}

#[derive(Debug)]
struct MappedRegion {
    device: Devices,
    start: Word,
    end: Word,
    mapping_mode: MappingMode,
}

#[derive(Debug, Default)]
pub struct MemoryMapper {
    regions: VecDeque<MappedRegion>,
}

impl MemoryMapper {
    pub fn map<W, D>(&mut self, device: D, start: W, end: W, mapping_mode: MappingMode) -> Result<()>
    where
        W: Into<Word>,
        D: Into<Devices>,
    {
        self.regions.push_front(MappedRegion {
            device: device.into(),
            start: start.into(),
            end: end.into(),
            mapping_mode,
        });
        Ok(())
    }

    fn find_region(&self, address: Word) -> Option<&MappedRegion> {
        self.regions
            .iter()
            .find(|region| address >= region.start && address <= region.end)
    }

    fn find_region_mut(&mut self, address: Word) -> Option<&mut MappedRegion> {
        self.regions
            .iter_mut()
            .find(|region| address >= region.start && address <= region.end)
    }
}

impl Addressable for MemoryMapper {
    fn read(&self, address: Word) -> Result<u8> {
        let Some(region) = self.find_region(address) else {
            return Err(Error::UnmappedAddress(address));
        };
        let address = match region.mapping_mode {
            MappingMode::Remap => address - region.start,
            MappingMode::Direct => address,
        };
        region.device.read(address)
    }

    fn write(&mut self, address: Word, byte: u8) -> Result<()> {
        let Some(region) = self.find_region_mut(address) else {
            return Err(Error::UnmappedAddress(address));
        };
        let address = match region.mapping_mode {
            MappingMode::Remap => address - region.start,
            MappingMode::Direct => address,
        };
        region.device.write(address, byte)
    }

    fn read_word(&self, address: Word) -> Result<u16> {
        let Some(region) = self.find_region(address) else {
            return Err(Error::UnmappedAddress(address));
        };
        let address = match region.mapping_mode {
            MappingMode::Remap => address - region.start,
            MappingMode::Direct => address,
        };
        region.device.read_word(address)
    }

    fn write_word(&mut self, address: Word, byte: u16) -> Result<()> {
        let Some(region) = self.find_region_mut(address) else {
            return Err(Error::UnmappedAddress(address));
        };
        let address = match region.mapping_mode {
            MappingMode::Remap => address - region.start,
            MappingMode::Direct => address,
        };
        region.device.write_word(address, byte)
    }
}
