use std::{env, fs};

use aya_core::bytecode::Loader;
use aya_core::cpu::Cpu;
use aya_core::memory::{Addressable, LinearMemory};
use aya_core::register::Register;
use aya_core::MEMORY_SIZE;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: WRITE TESTS RATHER THAN THIS

    let mut args = env::args();
    let filename = args.nth(1).expect("provide a program file");
    let content = fs::read_to_string(filename).expect("unable to read file");
    let program = Loader::load(content);

    let memory = LinearMemory::<MEMORY_SIZE>::default();
    let mut cpu = Cpu::new(memory);
    for (idx, byte) in program.into_iter().enumerate() {
        cpu.memory.write((idx as u16).try_into()?, byte)?;
    }

    let mut address = 0x3000;

    // set R1 to 0xFFFF
    cpu.memory.write_word(address.try_into()?, 0x0000)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0x0002)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0xFFFF)?;
    address += 2;

    // set R2 to 0xFFFF
    cpu.memory.write_word(address.try_into()?, 0x0000)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0x0003)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0xFFFF)?;
    address += 2;

    // set R3 to 0xFFFF
    cpu.memory.write_word(address.try_into()?, 0x0000)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0x0004)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0xFFFF)?;
    address += 2;

    // set R4 to 0xFFFF
    cpu.memory.write_word(address.try_into()?, 0x0000)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0x0005)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0xFFFF)?;
    address += 2;

    // set R5 to 0xFFFF
    cpu.memory.write_word(address.try_into()?, 0x0000)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0x0006)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0xFFFF)?;
    address += 2;

    // set R6 to 0xFFFF
    cpu.memory.write_word(address.try_into()?, 0x0000)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0x0007)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0xFFFF)?;
    address += 2;

    // set R7 to 0xFFFF
    cpu.memory.write_word(address.try_into()?, 0x0000)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0x0008)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0xFFFF)?;
    address += 2;

    // set R8 to 0xFFFF
    cpu.memory.write_word(address.try_into()?, 0x0000)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0x0009)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0xFFFF)?;
    address += 2;

    // set RET to 0xFEDC
    cpu.memory.write_word(address.try_into()?, 0x0000)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0x0000)?;
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0xFEDC)?;

    // RETURN
    address += 2;
    cpu.memory.write_word(address.try_into()?, 0x0011)?;

    for _ in 0..(10 + 10) {
        #[cfg(debug_assertions)]
        dump_memory(&cpu)?;
        cpu.step()?;
    }
    #[cfg(debug_assertions)]
    dump_memory(&cpu)?;
    Ok(())
}

#[cfg(debug_assertions)]
fn dump_memory<const T: usize>(cpu: &Cpu<T, LinearMemory<T>>) -> Result<(), Box<dyn std::error::Error>> {
    cpu.registers.inspect();
    cpu.memory.inspect_address(cpu.registers.fetch_word(Register::SP), 40)?;
    Ok(())
}
