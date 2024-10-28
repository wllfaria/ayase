use aya_bitmap::{Bitmap, Color};

use super::error::{Error, Result};

static PALETTES: &[(u8, u8, u8)] = &[
    // PALETTE 1
    (0x00, 0x00, 0x00),
    (0xb3, 0x00, 0x00),
    (0xff, 0x80, 0x00),
    (0xff, 0xff, 0xaa),
    (0x6c, 0xd9, 0x00),
    (0x00, 0x80, 0x00),
    (0x40, 0x40, 0x80),
    (0x88, 0x88, 0x88),
    // PALETTE 2
    (0x00, 0x00, 0x00),
    (0x6e, 0xb8, 0xa8),
    (0x2a, 0x58, 0x4f),
    (0x74, 0xa3, 0x3f),
    (0xfc, 0xff, 0xc0),
    (0xc6, 0x50, 0x5a),
    (0x77, 0x44, 0x48),
    (0xee, 0x9c, 0x5d),
];

pub fn compile_sprites(sprites: Vec<Bitmap>) -> Result<Vec<u8>> {
    let mut compiled = vec![];

    for sprite in sprites {
        let mut iter = sprite.data().iter().enumerate();

        while let Some((idx, color)) = iter.next() {
            let Some(left_idx) = PALETTES.iter().position(|c| Color::from(c) == *color) else {
                return Err(unknown_color(&sprite, color, idx));
            };

            let left_palette = (left_idx / 8) as u8;
            let left_idx = left_idx as u8 - 8 * left_palette;
            let mut packed: u8 = (left_palette << 3 | left_idx) << 4;

            let (idx, color) = iter.next().expect("sprites must have even number of pixels");
            let Some(right_idx) = PALETTES.iter().position(|c| Color::from(c) == *color) else {
                return Err(unknown_color(&sprite, color, idx));
            };

            let right_palette = (right_idx / 8) as u8;
            let right_idx = right_idx as u8 - 8 * right_palette;
            packed |= right_palette << 3 | right_idx;

            compiled.push(packed)
        }
    }

    if compiled.len() > 1024 * 4 {
        return Err(Error::SpriteTooBig(format!(
            "sprites should take at most 4KiB, but the total size is {}",
            compiled.len()
        )));
    }

    Ok(compiled)
}

fn unknown_color(sprite: &Bitmap, color: &Color, idx: usize) -> Error {
    let name = sprite.file_name();
    let width = sprite.info_header().width();
    let x = idx as u32 % width;
    let y = idx as u32 / width;

    Error::UnknownColor(format!(
        "color: {color} is not a valid palette color, found on sprite image: {name} at ({x}, {y})",
    ))
}
