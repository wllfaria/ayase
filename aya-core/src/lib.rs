pub mod bytecode;
pub mod cpu;
pub mod instruction;
pub mod memory;
pub mod op_code;
pub mod register;
mod word;

pub const MEMORY_SIZE: usize = u16::MAX as usize;
