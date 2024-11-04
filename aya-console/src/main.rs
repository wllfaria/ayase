mod renderer;
mod rom_loader;

use aya_console::memory::memory_mapper::{MappingMode, MemoryMapper};
use aya_console::memory::{
    BackgroundMemory, InterfaceMemory, ProgramMemory, SpriteMemory, StackMemory, TileMemory, BG_MEM_LOC, CODE_MEM_LOC,
    SPRITE_MEM_LOC, STACK_MEM_LOC, TILE_MEM_LOC, UI_MEM_LOC,
};
use aya_cpu::cpu::{ControlFlow, Cpu};
use aya_cpu::memory::Addressable;
use renderer::{RaylibRenderer, Renderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_file = std::env::args().nth(1).unwrap();
    let rom_file = std::fs::read(rom_file).unwrap();
    let rom_file = rom_loader::load_from_file(&rom_file);

    let memory = setup_memory(&rom_file);
    let mut cpu = Cpu::new(memory, CODE_MEM_LOC.0, STACK_MEM_LOC.1, 0x1000);
    cpu.load_into_address(rom_file.code, CODE_MEM_LOC.0).unwrap();

    let fps = 60.0;
    let scale = 8;
    let mut renderer = RaylibRenderer::new(fps, scale);
    while !renderer.should_close() {
        if let ControlFlow::Halt(_) = cpu.step()? {
            return Ok(());
        };

        if renderer.should_draw() {
            renderer.draw_frame(&mut cpu.memory)?;
        }
    }

    Ok(())
}

fn setup_memory(rom: &rom_loader::Rom) -> impl Addressable {
    let mut memory_mapper = MemoryMapper::default();

    let tile_memory = TileMemory::new(rom.sprites);
    memory_mapper
        .map(tile_memory, TILE_MEM_LOC.0, TILE_MEM_LOC.1, MappingMode::Remap)
        .unwrap();

    let sprite_memory = SpriteMemory::default();
    memory_mapper
        .map(sprite_memory, SPRITE_MEM_LOC.0, SPRITE_MEM_LOC.1, MappingMode::Remap)
        .unwrap();

    let code_memory = ProgramMemory::default();
    memory_mapper
        .map(code_memory, CODE_MEM_LOC.0, CODE_MEM_LOC.1, MappingMode::Direct)
        .unwrap();

    let bg_memory = BackgroundMemory::default();
    memory_mapper
        .map(bg_memory, BG_MEM_LOC.0, BG_MEM_LOC.1 + 1, MappingMode::Remap)
        .unwrap();

    let ui_memory = InterfaceMemory::default();
    memory_mapper
        .map(ui_memory, UI_MEM_LOC.0, UI_MEM_LOC.1, MappingMode::Remap)
        .unwrap();

    let stack_memory = StackMemory::default();
    memory_mapper
        .map(stack_memory, STACK_MEM_LOC.0, STACK_MEM_LOC.1, MappingMode::Remap)
        .unwrap();

    memory_mapper
}
