mod background_memory;
mod interface_memory;
pub mod memory_mapper;
mod program_memory;
mod sprite_memory;
mod stack_memory;
mod tile_memory;

pub use background_memory::BackgroundMemory;
pub use interface_memory::InterfaceMemory;
pub use program_memory::ProgramMemory;
pub use sprite_memory::SpriteMemory;
pub use stack_memory::StackMemory;
pub use tile_memory::TileMemory;

const KB: usize = 1024;
const KB8: usize = KB * 8;
const KB16: usize = KB * 16;

pub const TILE_MEMORY: usize = KB8;
pub const SPRITE_MEMORY: usize = 640;
pub const CODE_MEMORY: usize = KB16;
pub const BG_MEMORY: usize = KB;
pub const INTERFACE_MEMORY: usize = KB;
pub const STACK_MEMORY: usize = KB8;

pub const TILE_MEM_LOC: (u16, u16) = (0x0000, 0x1FFF);
pub const SPRITE_MEM_LOC: (u16, u16) = (0x2000, 0x227F);
pub const CODE_MEM_LOC: (u16, u16) = (0x2280, 0x627F);
pub const BG_MEM_LOC: (u16, u16) = (0x6280, 0x667F);
pub const UI_MEM_LOC: (u16, u16) = (0x6680, 0x6A7F);
pub const STACK_MEM_LOC: (u16, u16) = (0xE000, 0xFFFF);
