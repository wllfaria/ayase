mod input;
mod renderer;
mod rom_loader;

use std::path::Path;

use aya_cpu::cpu::{ControlFlow, Cpu};
use aya_cpu::memory::Addressable;
use input::{Input, KeyStatus, RaylibInput};
use memory::memory_mapper::{
    BackgroundMem, InputMem, InterfaceMem, InterruptMem, MappingMode, MemoryMapper, ProgramMem, SpriteMem, StackMem,
    TileMem,
};
use memory::{
    Interrupt, LinearMemory, BG_MEMORY, BG_MEM_LOC, CODE_MEMORY, CODE_MEM_LOC, INPUT_MEMORY, INPUT_MEM_LOC,
    INTERFACE_MEMORY, INTERRUPT_MEMORY, INTERRUPT_MEM_LOC, SPRITE_MEMORY, SPRITE_MEM_LOC, STACK_MEM_LOC, TILE_MEMORY,
    TILE_MEM_LOC, UI_MEM_LOC,
};
use renderer::{RaylibRenderer, Renderer};

const CLOCK_CYCLE: usize = 2000;
const FPS: f32 = 60.0;

pub mod memory;

pub static PALETTE: &[(u8, u8, u8, u8)] = &[
    (0x00, 0x00, 0x00, 0x00),
    (0x9d, 0xc1, 0xc0, 0xff),
    (0x52, 0x5b, 0x80, 0xff),
    (0x31, 0x21, 0x39, 0xff),
    (0x12, 0x0e, 0x1f, 0xff),
    (0x28, 0x46, 0x46, 0xff),
    (0x62, 0xab, 0x46, 0xff),
    (0x95, 0x53, 0x3d, 0xff),
    (0x6a, 0x24, 0x35, 0xff),
    (0x65, 0x41, 0x47, 0xff),
    (0xff, 0xf1, 0x69, 0xff),
    (0xd7, 0x79, 0x3f, 0xff),
    (0xab, 0x32, 0x29, 0xff),
    (0x9e, 0x8f, 0x84, 0xff),
    (0xe0, 0xb5, 0x6d, 0xff),
    (0xf6, 0x8b, 0x69, 0xff),
];

pub fn run<P: AsRef<Path>>(rom_file: P) -> Result<(), Box<dyn std::error::Error>> {
    let rom_file = std::fs::read(rom_file).unwrap();
    let rom_file = rom_loader::load_from_file(&rom_file);

    let memory = setup_memory(&rom_file);
    let mut cpu = Cpu::new(memory, CODE_MEM_LOC.0, STACK_MEM_LOC.1, INTERRUPT_MEM_LOC.0);
    cpu.load_into_address(rom_file.code, CODE_MEM_LOC.0).unwrap();

    let scale = 4;
    let mut renderer = RaylibRenderer::start(rom_file.name, FPS, scale);

    renderer.draw_frame(&mut cpu.memory)?;

    while !renderer.should_close() {
        let key_status = RaylibInput.poll();
        cpu.memory.write(INPUT_MEM_LOC.0, key_status)?;

        if renderer.should_draw() {
            renderer.draw_frame(&mut cpu.memory)?;
        }

        for _ in 0..CLOCK_CYCLE {
            if let ControlFlow::Halt(_) = cpu.step()? {
                return Ok(());
            };
        }

        cpu.memory.write(INPUT_MEM_LOC.0, KeyStatus::reset())?;
        cpu.handle_interrupt(Interrupt::AfterFrame)?;
    }

    Ok(())
}

fn setup_memory(rom: &rom_loader::Rom) -> impl Addressable {
    let mut memory_mapper = MemoryMapper::default();

    let tile_memory = LinearMemory::<TILE_MEMORY>::from(rom.sprites);
    memory_mapper
        .map(
            TileMem::from(tile_memory),
            TILE_MEM_LOC.0,
            TILE_MEM_LOC.1,
            MappingMode::Remap,
        )
        .unwrap();

    let sprite_memory = LinearMemory::<SPRITE_MEMORY>::default();
    memory_mapper
        .map(
            SpriteMem::from(sprite_memory),
            SPRITE_MEM_LOC.0,
            SPRITE_MEM_LOC.1,
            MappingMode::Remap,
        )
        .unwrap();

    let code_memory = LinearMemory::<CODE_MEMORY>::default();
    memory_mapper
        .map(
            ProgramMem::from(code_memory),
            CODE_MEM_LOC.0,
            CODE_MEM_LOC.1,
            MappingMode::Direct,
        )
        .unwrap();

    let bg_memory = LinearMemory::<BG_MEMORY>::default();
    memory_mapper
        .map(
            BackgroundMem::from(bg_memory),
            BG_MEM_LOC.0,
            BG_MEM_LOC.1 + 1,
            MappingMode::Remap,
        )
        .unwrap();

    let ui_memory = LinearMemory::<INTERFACE_MEMORY>::default();
    memory_mapper
        .map(
            InterfaceMem::from(ui_memory),
            UI_MEM_LOC.0,
            UI_MEM_LOC.1,
            MappingMode::Remap,
        )
        .unwrap();

    let interrupt_memory = LinearMemory::<INTERRUPT_MEMORY>::default();
    memory_mapper
        .map(
            InterruptMem::from(interrupt_memory),
            INTERRUPT_MEM_LOC.0,
            INTERRUPT_MEM_LOC.1,
            MappingMode::Remap,
        )
        .unwrap();

    let input_memory = LinearMemory::<INPUT_MEMORY>::default();
    memory_mapper
        .map(
            InputMem::from(input_memory),
            INPUT_MEM_LOC.0,
            INPUT_MEM_LOC.1,
            MappingMode::Remap,
        )
        .unwrap();

    let stack_memory = LinearMemory::default();
    memory_mapper
        .map(
            StackMem::from(stack_memory),
            STACK_MEM_LOC.0,
            STACK_MEM_LOC.1,
            MappingMode::Remap,
        )
        .unwrap();

    memory_mapper
}
