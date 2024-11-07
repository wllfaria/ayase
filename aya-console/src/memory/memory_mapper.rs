use std::collections::VecDeque;

use aya_cpu::memory::{Addressable, Error, Result};
use aya_cpu::word::Word;

use super::{
    LinearMemory, BG_MEMORY, CODE_MEMORY, INPUT_MEMORY, INTERFACE_MEMORY, INTERRUPT_MEMORY, SPRITE_MEMORY,
    STACK_MEMORY, TILE_MEMORY,
};

macro_rules! device {
    ($name:ident, $size:expr) => {
        #[derive(Debug)]
        pub struct $name(LinearMemory<$size>);

        impl From<LinearMemory<$size>> for $name {
            fn from(mem: LinearMemory<$size>) -> Self {
                Self(mem)
            }
        }

        impl Addressable for $name {
            fn write<W>(&mut self, address: W, byte: impl Into<u8>) -> Result<()>
            where
                W: Into<Word> + Copy,
            {
                self.0.write(address, byte)
            }

            fn read<W>(&self, address: W) -> Result<u8>
            where
                W: Into<Word> + Copy,
            {
                self.0.read(address)
            }

            fn write_word<W>(&mut self, address: W, word: u16) -> Result<()>
            where
                W: Into<Word> + Copy,
            {
                self.0.write_word(address, word)
            }

            fn read_word<W>(&self, address: W) -> Result<u16>
            where
                W: Into<Word> + Copy,
            {
                self.0.read_word(address)
            }
        }
    };
}

device!(TileMem, TILE_MEMORY);
device!(SpriteMem, SPRITE_MEMORY);
device!(ProgramMem, CODE_MEMORY);
device!(BackgroundMem, BG_MEMORY);
device!(InterfaceMem, INTERFACE_MEMORY);
device!(InterruptMem, INTERRUPT_MEMORY);
device!(InputMem, INPUT_MEMORY);
device!(StackMem, STACK_MEMORY);

macro_rules! devices {
    ($($variant:ident => $type:ty),* $(,)?) => {
        #[derive(Debug)]
        #[allow(clippy::large_enum_variant)]
        pub enum Devices {
            $($variant($type),)*
        }

        impl Addressable for Devices {
            fn write<W>(&mut self, address: W, byte: impl Into<u8>) -> Result<()>
            where
                W: Into<Word> + Copy,
            {
                match self {
                    $(Devices::$variant(mem) => mem.write(address, byte),)*
                }
            }

            fn read<W>(&self, address: W) -> Result<u8>
            where
                W: Into<Word> + Copy,
            {
                match self {
                    $(Devices::$variant(mem) => mem.read(address.into()),)*
                }
            }

            fn write_word<W>(&mut self, address: W, word: u16) -> Result<()>
            where
                W: Into<Word> + Copy,
            {
                match self {
                    $(Devices::$variant(mem) => mem.write_word(address, word),)*
                }
            }

            fn read_word<W>(&self, address: W) -> Result<u16>
            where
                W: Into<Word> + Copy,
            {
                match self {
                    $(Devices::$variant(mem) => mem.read_word(address),)*
                }
            }
        }

        $(impl From<$type> for Devices {
            fn from(mem: $type) -> Self {
                Self::$variant(mem)
            }
        })*
    };
}

devices! {
    Tile => TileMem,
    Sprite => SpriteMem,
    Program => ProgramMem,
    Background => BackgroundMem,
    Interface => InterfaceMem,
    Interrupt => InterruptMem,
    Input => InputMem,
    Stack => StackMem,
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
    fn read<W>(&self, address: W) -> Result<u8>
    where
        W: Into<Word> + Copy,
    {
        let address = address.into();
        let Some(region) = self.find_region(address) else {
            return Err(Error::UnmappedAddress(address));
        };
        let address = match region.mapping_mode {
            MappingMode::Remap => address - region.start,
            MappingMode::Direct => address,
        };
        region.device.read(address)
    }

    fn write<W>(&mut self, address: W, byte: impl Into<u8>) -> Result<()>
    where
        W: Into<Word> + Copy,
    {
        let address = address.into();
        let Some(region) = self.find_region_mut(address) else {
            return Err(Error::UnmappedAddress(address));
        };

        if region.end - region.start == 1.into() {
            println!("{:?}", region);
        }

        let address = match region.mapping_mode {
            MappingMode::Remap => address - region.start,
            MappingMode::Direct => address,
        };
        region.device.write(address, byte)
    }

    fn read_word<W>(&self, address: W) -> Result<u16>
    where
        W: Into<Word> + Copy,
    {
        let Some(region) = self.find_region(address.into()) else {
            return Err(Error::UnmappedAddress(address.into()));
        };
        let address = match region.mapping_mode {
            MappingMode::Remap => address.into() - region.start,
            MappingMode::Direct => address.into(),
        };
        region.device.read_word(address)
    }

    fn write_word<W>(&mut self, address: W, word: u16) -> Result<()>
    where
        W: Into<Word> + Copy,
    {
        let Some(region) = self.find_region_mut(address.into()) else {
            return Err(Error::UnmappedAddress(address.into()));
        };
        let address = match region.mapping_mode {
            MappingMode::Remap => address.into() - region.start,
            MappingMode::Direct => address.into(),
        };
        region.device.write_word(address, word)
    }
}
