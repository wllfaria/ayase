#[derive(Debug)]
pub struct Rom<'rom> {
    pub name: &'rom str,
    pub code: &'rom [u8],
    pub sprites: &'rom [u8],
}

pub fn load_from_file(rom: &[u8]) -> Rom {
    assert!(rom.len() > 128);
    assert!(&rom[0..3] == b"AYA");

    let name_len = rom[5..]
        .iter()
        .position(|ch| *ch == 0)
        .expect("no null terminator after name");
    let name = std::str::from_utf8(&rom[5..5 + name_len]).unwrap();

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

    println!("{code:02X?}");

    Rom { name, code, sprites }
}
