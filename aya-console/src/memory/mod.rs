mod memory_mapper;
mod program_memory;
mod sprite_memory;
mod stack_memory;
mod stdout_memory;
mod video_memory;

pub use memory_mapper::{MappingMode, MemoryMapper};
pub use program_memory::ProgramMemory;
pub use sprite_memory::SpriteMemory;
pub use stack_memory::StackMemory;
pub use stdout_memory::OutputMemory as StdoutMemory;
pub use video_memory::VideoMemory;

const ONEKB: usize = 1024;
const KB4: usize = ONEKB * 4;
const KB8: usize = ONEKB * 8;
const KB16: usize = ONEKB * 16;
