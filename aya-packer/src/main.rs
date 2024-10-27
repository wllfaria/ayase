mod config;
mod error;
use std::path::PathBuf;

use aya_bitmap::{Bitmap, Color};
use clap::Parser;
use config::Config;
use error::Result;

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

static CONFIG_FILE: &str = "aya.cfg";

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, required = false, long, requires = "sprites", requires = "name")]
    code: Option<String>,

    #[arg(short, required = false, long, requires = "code", requires = "name")]
    sprites: Option<Vec<String>>,

    #[arg(short, required = false, long, requires = "code", requires = "sprites")]
    name: Option<String>,

    #[arg(short, required = false, long)]
    output: Option<String>,

    #[arg(long, required = false)]
    config: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let config = match args.code.is_some() {
        true => Config::from_args(args),
        false => config::read_from_file(args.config.unwrap_or(CONFIG_FILE.into())).expect("failed to read config file"),
    };

    let code = PathBuf::from(&config.code);
    let code = aya_assembly::compile(&code);

    let sprites = config.sprites.iter().map(PathBuf::from).collect::<Vec<_>>();
    let sprites = sprites
        .into_iter()
        .map(aya_bitmap::decode)
        .map(|r| r.expect("image was not a bitmap"))
        .collect::<Vec<_>>();
    let sprites = compile_sprites(sprites).unwrap();

    let header = make_header(&config, code.len() as u16, sprites.len() as u16);

    let mut rom = vec![];
    rom.extend(header);
    rom.extend(code);
    rom.extend(sprites);

    std::fs::write(config.output, rom).unwrap();

    Ok(())
}

fn make_header(config: &Config, code_size: u16, sprite_size: u16) -> Vec<u8> {
    const HEADER_SIZE: usize = 128;
    let mut header = vec![0; HEADER_SIZE];

    // SIGNATURE
    header[0x00] = b'A';
    header[0x01] = b'Y';
    header[0x02] = b'A';

    // VERSION
    header[0x04] = 1;

    // GAME NAME
    assert!(config.name.len() <= 63);
    for (i, c) in config.name.chars().enumerate() {
        header[0x05 + i] = c as u8;
    }

    header[0x44] = 0x80;
    header[0x45] = 0x00;

    let [lower, upper] = u16::to_le_bytes(code_size);
    header[0x46] = lower;
    header[0x47] = upper;

    let [lower, upper] = u16::to_le_bytes(code_size + HEADER_SIZE as u16);
    header[0x48] = lower;
    header[0x49] = upper;

    let [lower, upper] = u16::to_le_bytes(sprite_size);
    header[0x4A] = lower;
    header[0x4B] = upper;

    header
}

fn compile_sprites(sprites: Vec<Bitmap>) -> Result<Vec<u8>> {
    let mut compiled = vec![];

    for sprite in sprites {
        let mut iter = sprite.data.into_iter().peekable();

        while let Some(color) = iter.next() {
            let left_index = PALETTES
                .iter()
                .position(|c| Color::from(c) == color)
                .expect("color on bitmap is not within the palette");
            let left_palette = if (left_index as u8) < 8 { 0 } else { 1 };
            let left_index = left_index as u8 - 8 * left_palette;
            let mut packed: u8 = (left_palette << 3 | left_index) << 4;

            let color = iter.next().expect("sprites must have even number of pixels");

            let right_index = PALETTES
                .iter()
                .position(|c| Color::from(c) == color)
                .expect("color on bitmap is not within the palette");
            let right_palette = if (right_index as u8) < 8 { 0 } else { 1 };
            let right_index = right_index as u8 - 8 * right_palette;
            packed |= right_palette << 3 | right_index;

            compiled.push(packed)
        }
    }

    assert!(compiled.len() < 1024 * 4, "sprites should take up to 4KiB");

    Ok(compiled)
}
