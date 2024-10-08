pub mod bytecode;
pub mod cpu;
mod instruction;
pub mod memory;
mod op_code;
pub mod register;
mod word;

pub const MEMORY_SIZE: usize = u16::MAX as usize;
