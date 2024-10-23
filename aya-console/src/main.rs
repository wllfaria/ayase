pub mod memory;
use aya_core::cpu::Cpu;
use aya_core::memory::Addressable;
use aya_core::MEMORY_SIZE;
use memory::{LinearMemory, MappingMode, MemoryMapper, OutputDevice};
use raylib::prelude::*;

fn main() {
    let mut memory_mapper = MemoryMapper::<MEMORY_SIZE>::default();

    let code_memory = LinearMemory::<MEMORY_SIZE>::default();
    memory_mapper
        .map(code_memory, 0x0000, 0x7FFF, MappingMode::Remap)
        .unwrap();

    let screen_device = OutputDevice::default();
    memory_mapper
        .map(screen_device, 0x8000, 0xFFFF, MappingMode::Remap)
        .unwrap();

    let mut cpu = Cpu::new(memory_mapper);
    let file = std::env::args().nth(1).unwrap();
    let bytecode = aya_compiler::compile(file);
    cpu.load_into_address(bytecode, 0x0000).unwrap();

    let (mut rl, thread) = raylib::init().size(640, 640).undecorated().title("Hello").build();
    while !rl.window_should_close() {
        cpu.step().unwrap();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_rectangle(40, 40, 40, 40, Color::BISQUE);
    }
}
