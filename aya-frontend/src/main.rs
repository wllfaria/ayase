use std::env;
use std::fs;

use aya_core::bytecode::Loader;
use aya_core::cpu::Cpu;
use aya_core::memory::{Addressable, LinearMemory};
use aya_core::register::Register;
use aya_core::MEMORY_SIZE;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let filename = args.nth(1).expect("provide a program file");
    let content = fs::read_to_string(&filename).expect("unable to read file");
    let program = Loader::load(&content);

    let memory = LinearMemory::<MEMORY_SIZE>::default();
    let mut cpu = Cpu::new(memory);
    for (idx, byte) in program.into_iter().enumerate() {
        cpu.memory.write((idx as u16).into(), byte)?;
    }

    cpu.step().ok();
    dump_memory(&cpu);
    Ok(())
}

fn dump_memory<const T: usize>(cpu: &Cpu<T, LinearMemory<T>>) {
    for register in Register::iter() {
        println!("{: <3} @ 0x{:04X}", register, cpu.registers.fetch(register));
    }
    cpu.memory.inspect_address(0.into(), 40).ok();
}
