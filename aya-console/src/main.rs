mod rom_loader;

use std::time::{Duration, Instant};

use aya_console::memory::memory_mapper::{MappingMode, MemoryKind, MemoryMapper};
use aya_console::memory::{
    BackgroundMemory, InterfaceMemory, ProgramMemory, SpriteMemory, StackMemory, TileMemory, BG_MEM_LOC, CODE_MEM_LOC,
    SPRITE_MEM_LOC, STACK_MEM_LOC, TILE_MEM_LOC, UI_MEM_LOC,
};
use aya_console::PALETTE;
use aya_cpu::cpu::{ControlFlow, Cpu};
use aya_cpu::memory::Addressable;
use raylib::prelude::*;

//const FREQUENCY: f64 = 4_200_000.0;
//const CYCLES_PER_FRAME: u64 = (FREQUENCY / FPS) as u64;
const FPS: f64 = 30.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_file = std::env::args().nth(1).unwrap();
    let rom_file = std::fs::read(rom_file).unwrap();
    let rom_file = rom_loader::load_from_file(&rom_file);

    let memory = setup_memory(&rom_file);
    let mut cpu = Cpu::new(memory, 0x2280, 0xFFFF);
    cpu.load_into_address(rom_file.code, 0x2280).unwrap();

    let mut frame_start = Instant::now();
    let frame_duration = Duration::from_secs_f64(1.0 / FPS);

    let scale = 3u16;

    let (mut rl, thread) = raylib::init()
        .undecorated()
        .size(256 * scale as i32, 256 * scale as i32)
        .title("Hello")
        .resizable()
        .build();

    while !rl.window_should_close() {
        if let ControlFlow::Halt(_) = cpu.step()? {
            return Ok(());
        };

        if frame_start.elapsed() >= frame_duration {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);

            for bg_idx in 0..32 * 32 {
                let tile_idx = cpu.memory.read(BG_MEM_LOC.0 + bg_idx)?;
                let tile_x = bg_idx % 32 * 8 * scale;
                let tile_y = bg_idx / 32 * 8 * scale;
                let start = TILE_MEM_LOC.0 + tile_idx as u16 * 32;

                for byte_idx in 0..32 {
                    let tile_byte = cpu.memory.read(start + byte_idx)?;
                    let color_left = PALETTE[(tile_byte >> 4) as usize];
                    let color_right = PALETTE[(tile_byte & 0xf) as usize];

                    let left_x = tile_x + ((byte_idx % 4) * 2) * scale;
                    let right_x = left_x + scale;
                    let y = tile_y + byte_idx / 4 * scale;

                    d.draw_rectangle(
                        left_x as i32,
                        y as i32,
                        scale as i32,
                        scale as i32,
                        Color::from(color_left),
                    );

                    d.draw_rectangle(
                        right_x as i32,
                        y as i32,
                        scale as i32,
                        scale as i32,
                        Color::from(color_right),
                    );
                }
            }

            frame_start = Instant::now();
        }
    }

    Ok(())
}

fn setup_memory(rom: &rom_loader::Rom) -> impl Addressable {
    let mut memory_mapper = MemoryMapper::default();

    let tile_memory = TileMemory::new(rom.sprites);
    memory_mapper
        .map(
            tile_memory,
            TILE_MEM_LOC.0,
            TILE_MEM_LOC.1,
            MappingMode::Remap,
            MemoryKind::Readonly,
        )
        .unwrap();

    let sprite_memory = SpriteMemory::default();
    memory_mapper
        .map(
            sprite_memory,
            SPRITE_MEM_LOC.0,
            SPRITE_MEM_LOC.1,
            MappingMode::Remap,
            MemoryKind::Readonly,
        )
        .unwrap();

    let code_memory = ProgramMemory::default();
    memory_mapper
        .map(
            code_memory,
            CODE_MEM_LOC.0,
            CODE_MEM_LOC.1,
            MappingMode::Direct,
            MemoryKind::Readonly,
        )
        .unwrap();

    let bg_memory = BackgroundMemory::default();
    memory_mapper
        .map(
            bg_memory,
            BG_MEM_LOC.0,
            BG_MEM_LOC.1 + 1,
            MappingMode::Remap,
            MemoryKind::ReadWrite,
        )
        .unwrap();

    let ui_memory = InterfaceMemory::default();
    memory_mapper
        .map(
            ui_memory,
            UI_MEM_LOC.0,
            UI_MEM_LOC.1,
            MappingMode::Remap,
            MemoryKind::Readonly,
        )
        .unwrap();

    let stack_memory = StackMemory::default();
    memory_mapper
        .map(
            stack_memory,
            STACK_MEM_LOC.0,
            STACK_MEM_LOC.1,
            MappingMode::Remap,
            MemoryKind::ReadWrite,
        )
        .unwrap();

    memory_mapper
}
