mod error;
mod header;
mod sprites;

pub use header::make_header;
pub use sprites::compile_sprites;

pub fn compile(header: &[u8], code: &[u8], sprites: &[u8]) -> Vec<u8> {
    let mut rom = vec![];
    rom.extend(header);
    rom.extend(code);
    rom.extend(sprites);
    rom
}
