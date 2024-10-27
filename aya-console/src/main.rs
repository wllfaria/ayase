use aya_console::memory::{
    MappingMode, MemoryMapper, ProgramMemory, SpriteMemory, StackMemory, StdoutMemory, VideoMemory,
};
use aya_cpu::cpu::Cpu;
//use raylib::prelude::*;

fn main() {
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

    let stdout = StdoutMemory::default();
    memory_mapper.map(stdout, 0x8000, 0x9FFF, MappingMode::Remap).unwrap();

    let stack_memory = StackMemory::default();
    memory_mapper
        .map(stack_memory, 0xE000, 0xFFFF, MappingMode::Remap)
        .unwrap();

    let mut cpu = Cpu::new(memory_mapper);
    let rom_file = std::env::args().nth(1).unwrap();
    let rom_file = std::fs::read(rom_file).unwrap();
    let rom_file = parse_rom_file(&rom_file);

    cpu.load_into_address(rom_file.code, 0x0000).unwrap();
    cpu.load_into_address(rom_file.sprites, 0x4000).unwrap();
    cpu.run();

    // TODO
    // we need to emulate a clock rate for the VM
    // we need to read from graphics memory and update the graphics context
    // we need interrupts to handle player controls

    //let (mut rl, thread) = raylib::init().size(640, 640).undecorated().title("Hello").build();
    //while !rl.window_should_close() {
    //    let mut d = rl.begin_drawing(&thread);
    //    d.clear_background(Color::BLACK);
    //    d.draw_rectangle(40, 40, 40, 40, Color::BISQUE);
    //}
}

#[derive(Debug)]
struct Rom<'rom> {
    code: &'rom [u8],
    sprites: &'rom [u8],
}

fn parse_rom_file(rom: &[u8]) -> Rom {
    assert!(rom.len() > 128);
    assert!(&rom[0..3] == b"AYA");

    //let name_len = rom[5..]
    //    .iter()
    //    .position(|ch| *ch == 0)
    //    .expect("no null terminator after name");
    //let name = std::str::from_utf8(&rom[5..name_len]).unwrap();

    let code_offset: [u8; 2] = rom[0x44..0x46].try_into().unwrap();
    let code_offset = u16::from_le_bytes(code_offset) as usize;
    let code_size: [u8; 2] = rom[0x46..0x48].try_into().unwrap();
    let code_size = u16::from_le_bytes(code_size) as usize;

    let sprites_offset: [u8; 2] = rom[0x48..0x4A].try_into().unwrap();
    let sprites_offset = u16::from_le_bytes(sprites_offset) as usize;
    let sprites_size: [u8; 2] = rom[0x4A..0x4C].try_into().unwrap();
    let sprites_size = u16::from_le_bytes(sprites_size) as usize;

    let code = &rom[code_offset..code_offset + code_size];
    let sprites = &rom[sprites_offset..sprites_offset + sprites_size];

    Rom { code, sprites }
}
