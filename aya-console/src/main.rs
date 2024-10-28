mod rom_loader;

use std::time::{Duration, Instant};

use aya_console::memory::{MappingMode, MemoryMapper, ProgramMemory, SpriteMemory, StackMemory, VideoMemory};
use aya_cpu::cpu::{ControlFlow, Cpu};
use aya_cpu::memory::Addressable;
use raylib::prelude::*;

const FREQUENCY: f64 = 4_200_000.0;
const FPS: f64 = 30.0;
const CYCLES_PER_FRAME: u64 = (FREQUENCY / FPS) as u64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = setup_memory();
    let mut cpu = Cpu::new(memory);
    let rom_file = std::env::args().nth(1).unwrap();
    let rom_file = std::fs::read(rom_file).unwrap();
    let rom_file = rom_loader::load_from_file(&rom_file);

    cpu.load_into_address(rom_file.code, 0x0000).unwrap();
    cpu.load_into_address(rom_file.sprites, 0x4000).unwrap();

    // TODO:
    // we need to emulate a clock rate for the VM
    // we need to read from graphics memory and update the graphics context
    // we need interrupts to handle player controls

    let mut frame_start = Instant::now();
    let frame_duration = Duration::from_secs_f64(1.0 / FPS);

    let (mut rl, thread) = raylib::init().size(256, 256).undecorated().title("Hello").build();
    while !rl.window_should_close() {
        for _ in 0..CYCLES_PER_FRAME {
            if let ControlFlow::Halt(_) = cpu.step()? {
                return Ok(());
            };
        }

        if frame_start.elapsed() >= frame_duration {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);
            d.draw_rectangle(40, 40, 40, 40, Color::BISQUE);
            frame_start = Instant::now();
        }
    }

    Ok(())
}

fn setup_memory() -> impl Addressable {
    let mut memory_mapper = MemoryMapper::default();

    let code_memory = ProgramMemory::default();
    memory_mapper
        .map(code_memory, 0x0000, 0x3FFF, MappingMode::Remap)
        .unwrap();

    let sprite_memory = SpriteMemory::default();
    memory_mapper
        .map(sprite_memory, 0x4000, 0x4FFF, MappingMode::Remap)
        .unwrap();

    let screen_device = VideoMemory::default();
    memory_mapper
        .map(screen_device, 0x5000, 0x6FFF, MappingMode::Remap)
        .unwrap();

    let stack_memory = StackMemory::default();
    memory_mapper
        .map(stack_memory, 0xE000, 0xFFFF, MappingMode::Remap)
        .unwrap();

    memory_mapper
}
