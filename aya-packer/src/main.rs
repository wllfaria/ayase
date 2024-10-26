mod config;
mod error;
use std::path::PathBuf;

use clap::Parser;
use error::Result;

static CONFIG_FILE: &str = "aya.cfg";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, required = false, long, requires = "sprite")]
    code: Option<String>,

    #[arg(short, required = false, long, requires = "code")]
    sprite: Option<Vec<String>>,

    #[arg(long, required = false)]
    config: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (code, sprites) = match args.code.is_some() {
        true => (args.code.unwrap(), args.sprite.unwrap()),
        false => match config::read_from_file(args.config.unwrap_or(CONFIG_FILE.into())) {
            Ok(config) => (config.code, config.sprites),
            Err(_) => (config::default().code, config::default().sprites),
        },
    };

    let code = PathBuf::from(code);
    let sprites = sprites.into_iter().map(PathBuf::from).collect::<Vec<_>>();

    let code = aya_assembly::compile(&code);
    let sprites = sprites
        .into_iter()
        .map(aya_bitmap::decode)
        .map(|r| r.unwrap())
        .collect::<Vec<_>>();

    println!("{sprites:#?}");

    Ok(())
}
