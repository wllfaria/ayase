use std::{env, fs};

use aya_core::cpu::Cpu;
use aya_core::instruction::Instruction;
use aya_core::memory::{Addressable, LinearMemory, MappingMode, MemoryMapper, OutputMemory};
use aya_core::MEMORY_SIZE;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: WRITE TESTS RATHER THAN THIS

    let mut args = env::args();
    let filename = args.nth(1).expect("provide a program file");
    let content = fs::read_to_string(filename).expect("unable to read file");
    let program = aya_compiler::compile(&content);

    let mut memory_mapper = MemoryMapper::<MEMORY_SIZE>::default();

    let memory = LinearMemory::<MEMORY_SIZE>::default();
    memory_mapper.map(memory, 0x0000, 0xffff, MappingMode::Remap)?;

    let output = OutputMemory::<MEMORY_SIZE>::default();
    memory_mapper.map(output, 0x3000, 0x30ff, MappingMode::Remap)?;

    let mut cpu = Cpu::new(memory_mapper);

    for (idx, byte) in program.into_iter().enumerate() {
        cpu.memory.write((idx as u16).try_into()?, byte)?;
    }

    cpu.run(dump_memory);
    cpu.run(|_, _| {});

    Ok(())
}

#[cfg(debug_assertions)]
fn dump_memory<const SIZE: usize, A: Addressable<SIZE>>(cpu: &mut Cpu<SIZE, A>, instruction: &Instruction<SIZE>) {
    println!("{instruction:?}");
    cpu.registers.inspect();
    cpu.memory.inspect_address(0x3000, 24).unwrap();
}
