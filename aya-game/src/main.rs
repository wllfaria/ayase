use aya_cpu::cpu::Cpu;
use aya_game::memory::{MappingMode, MemoryMapper, SpriteMemory, VideoMemory};
use raylib::prelude::*;

fn main() {
    let mut memory_mapper = MemoryMapper::default();

    let code_memory = SpriteMemory::default();
    memory_mapper
        .map(code_memory, 0x0000, 0x7FFF, MappingMode::Remap)
        .unwrap();

    let screen_device = VideoMemory::default();
    memory_mapper
        .map(screen_device, 0x8000, 0xFFFF, MappingMode::Remap)
        .unwrap();

    let mut cpu = Cpu::new(memory_mapper);
    let file = std::env::args().nth(1).unwrap();
    let bytecode = aya_compiler::compile(file);
    cpu.load_into_address(bytecode, 0x0000).unwrap();

    let (mut rl, thread) = raylib::init().size(640, 640).undecorated().title("Hello").build();
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_rectangle(40, 40, 40, 40, Color::BISQUE);
    }
}
