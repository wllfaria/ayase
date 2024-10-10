use std::{env, fs};

use aya_core::bytecode::Loader;
use aya_core::cpu::Cpu;
use aya_core::memory::{Addressable, LinearMemory, MappingMode, MemoryMapper, OutputMemory};
use aya_core::register::Register;
use aya_core::MEMORY_SIZE;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: WRITE TESTS RATHER THAN THIS

    let mut args = env::args();
    let filename = args.nth(1).expect("provide a program file");
    let content = fs::read_to_string(filename).expect("unable to read file");
    let program = Loader::load(content);

    let mut memory_mapper = MemoryMapper::<MEMORY_SIZE>::default();

    let memory = LinearMemory::<MEMORY_SIZE>::default();
    memory_mapper.map(memory, 0x0000, 0xffff, MappingMode::Remap)?;

    let output = OutputMemory::<MEMORY_SIZE>::default();
    memory_mapper.map(output, 0x3000, 0x30ff, MappingMode::Remap)?;

    let mut cpu = Cpu::new(memory_mapper);

    for (idx, byte) in program.into_iter().enumerate() {
        cpu.memory.write((idx as u16).try_into()?, byte)?;
    }

    cpu.run();

    println!();
    #[cfg(debug_assertions)]
    dump_memory(&mut cpu)?;
    Ok(())
}

#[cfg(debug_assertions)]
fn dump_memory<const SIZE: usize, A: Addressable<SIZE>>(
    cpu: &mut Cpu<SIZE, A>,
) -> Result<(), Box<dyn std::error::Error>> {
    cpu.registers.inspect();
    cpu.memory.inspect_address(cpu.registers.fetch_word(Register::SP), 40)?;
    Ok(())
}
