mod linear_memory;
pub mod memory_mapper;

pub use linear_memory::LinearMemory;

const KB: usize = 1024;
const KB8: usize = KB * 8;
const KB16: usize = KB * 16;

pub const TILE_MEMORY: usize = KB8;
pub const SPRITE_MEMORY: usize = 640;
pub const CODE_MEMORY: usize = KB16;
pub const BG_MEMORY: usize = 420;
pub const INTERFACE_MEMORY: usize = 420;
pub const INTERRUPT_MEMORY: usize = 16;
pub const INPUT_MEMORY: usize = 1;
pub const STACK_MEMORY: usize = KB8;

/// 8KIB Tile memory
pub const TILE_MEM_LOC: (u16, u16) = (0x0000, 0x1FFF);

/// 640B Sprite memory
pub const SPRITE_MEM_LOC: (u16, u16) = (0x2000, 0x227F);

/// 16KB Code memory
pub const CODE_MEM_LOC: (u16, u16) = (0x2280, 0x627F);

/// 420B Background memory
pub const BG_MEM_LOC: (u16, u16) = (0x6280, 0x6423);

/// 420B Foreground memory
pub const FG_MEM_LOC: (u16, u16) = (0x6424, 0x65C7);

/// 420B Interface memory
pub const UI_MEM_LOC: (u16, u16) = (0x65C8, 0x676B);

///  16B Interrupt table
pub const INTERRUPT_MEM_LOC: (u16, u16) = (0x676C, 0x677B);

///   1B Input mapping
pub const INPUT_MEM_LOC: (u16, u16) = (0x677C, 0x677C);

/// 8KiB Stack memory
pub const STACK_MEM_LOC: (u16, u16) = (0xE000, 0xFFFF);

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Interrupt {
    AfterFrame,
}

impl From<Interrupt> for u16 {
    fn from(value: Interrupt) -> Self {
        value as u16
    }
}
