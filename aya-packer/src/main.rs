mod config;
mod rom;

use std::path::PathBuf;
use std::process::ExitCode;

use aya_assembly::{AssembleBehavior, AssembleOutput};
use clap::Parser;
use config::Config;

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

    #[arg(short, required = false, long)]
    expand: Option<bool>,

    #[arg(long, required = false)]
    config: Option<String>,
}

fn main() -> std::result::Result<ExitCode, Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config = match args.code.is_some() {
        true => Config::from_args(args),
        false => config::read_from_file(args.config.unwrap_or(CONFIG_FILE.into()))
            .expect("unable to read config file. Please certify that a aya.cfg file exists in the current directory"),
    };

    let path = PathBuf::from(&config.code);

    let behavior = if config.expand { AssembleBehavior::Codegen } else { AssembleBehavior::Bytecode };

    let output = aya_assembly::assemble(&path, behavior)?;

    if config.expand {
        let AssembleOutput::Codegen(code) = output else {
            unreachable!();
        };
        std::fs::write(config.output, code).expect("failed to write expanded code into specified output");
        return Ok(ExitCode::FAILURE);
    }

    let AssembleOutput::Bytecode(code) = output else {
        unreachable!();
    };

    let mut sprites = vec![];
    let sprite_paths = config.sprites.iter().map(PathBuf::from).collect::<Vec<_>>();
    for path in sprite_paths {
        sprites.push(aya_bitmap::decode(path)?);
    }

    let sprites = match rom::compile_sprites(sprites) {
        Ok(sprites) => sprites,
        Err(rom::Error::SpriteTooBig(msg)) => {
            eprintln!("{msg}");
            return Ok(ExitCode::FAILURE);
        }
        Err(rom::Error::UnknownColor(msg)) => {
            eprintln!("{msg}");
            return Ok(ExitCode::FAILURE);
        }
    };
    let header = rom::make_header(&config, code.len() as u16, sprites.len() as u16);
    let rom = rom::compile(&header, &code, &sprites);

    std::fs::write(config.output, rom).expect("failed to write rom into specified output");

    Ok(ExitCode::SUCCESS)
}
