use aya_bitmap::{Bitmap, Color};
use aya_console::memory::TILE_MEMORY;
use aya_console::PALETTE;

use super::error::{Error, Result};

pub fn compile_sprites(sprites: Vec<Bitmap>) -> Result<Vec<u8>> {
    let mut compiled = vec![];

    for sprite in sprites {
        let width = sprite.info_header().width();
        let height = sprite.info_header().height();
        let data = sprite.data();

        if width % 8 != 0 || height % 8 != 0 {
            panic!("invalid sprite size");
        }

        let num_sprites_x = width / 8;
        let num_sprites_y = height / 8;

        for sprite_y in 0..num_sprites_y {
            for sprite_x in 0..num_sprites_x {
                for row in 0..8 {
                    for col in (0..8).step_by(2) {
                        let global_row = sprite_y * 8 + row;
                        let global_col = sprite_x * 8 + col;
                        let idx = (global_row * width + global_col) as usize;

                        let left_color = data[idx];
                        let right_color = data[idx + 1];

                        let Some(left_idx) = PALETTE
                            .iter()
                            .position(|&(r, g, b, _)| Color::from((r, g, b)) == left_color)
                        else {
                            return Err(unknown_color(&sprite, &left_color, idx));
                        };

                        let Some(right_idx) = PALETTE
                            .iter()
                            .position(|&(r, g, b, _)| Color::from((r, g, b)) == right_color)
                        else {
                            return Err(unknown_color(&sprite, &right_color, idx + 1));
                        };

                        let packed: u8 = (left_idx as u8) << 4 | (right_idx as u8);
                        compiled.push(packed);
                    }
                }
            }
        }
    }

    if compiled.len() > TILE_MEMORY {
        return Err(Error::SpriteTooBig(format!(
            "sprites should take at most {}KiB, but the total size is {}",
            TILE_MEMORY >> 10,
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
