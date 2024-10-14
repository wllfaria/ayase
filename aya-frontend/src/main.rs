use std::{env, fs};

use aya_core::cpu::Cpu;
use aya_core::memory::{LinearMemory, MappingMode, MemoryMapper, OutputMemory};
use aya_core::MEMORY_SIZE;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let filename = args.nth(1).expect("provide a program file");
    let content = fs::read_to_string(filename).expect("unable to read file");
    let program = aya_compiler::compile(&content);

    let memory = LinearMemory::<MEMORY_SIZE>::default();
    let output = OutputMemory::<MEMORY_SIZE>::default();

    let mut memory_mapper = MemoryMapper::<MEMORY_SIZE>::default();
    memory_mapper.map(memory, 0x0000, 0xffff, MappingMode::Remap)?;
    memory_mapper.map(output, 0x3000, 0x30ff, MappingMode::Remap)?;

    let mut cpu = Cpu::new(memory_mapper);
    cpu.load_into_address(program, 0x0000)?;
    cpu.run();

    Ok(())
}
