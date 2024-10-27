mod lexer;
mod parser;
use parser::Key;

use crate::error::{Error, Result};
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Config {
    pub code: String,
    pub sprites: Vec<String>,
    pub name: String,
    pub output: String,
}

impl Config {
    pub(crate) fn from_args(args: crate::Args) -> Self {
        Self {
            name: args.name.unwrap(),
            code: args.code.unwrap(),
            sprites: args.sprites.unwrap(),
            output: args.output.unwrap_or("a.out".into()),
        }
    }

    pub(crate) fn from_keys(source: &str, keys: Vec<Key>) -> Self {
        let code = extract_key(&keys, |key| {
            let Key::Code(offset) = key else {
                return None;
            };
            Some(*offset)
        });
        let code = source[std::ops::Range::<usize>::from(code)].to_string();

        let sprites = extract_key(&keys, |key| {
            let Key::Sprites(offsets) = key else {
                return None;
            };
            Some(offsets.clone())
        });

        let sprites = sprites
            .into_iter()
            .map(|offset| source[std::ops::Range::<usize>::from(offset)].to_string())
            .collect::<Vec<_>>();

        let name = extract_key(&keys, |key| {
            let Key::Name(offset) = key else {
                return None;
            };
            Some(*offset)
        });
        let name = source[std::ops::Range::<usize>::from(name)].to_string();

        let output = extract_key(&keys, |key| {
            let Key::Output(offset) = key else {
                return None;
            };
            Some(*offset)
        });
        let output = source[std::ops::Range::<usize>::from(output)].to_string();

        Self {
            code,
            sprites,
            name,
            output,
        }
    }
}

fn extract_key<T, F: FnMut(&Key) -> Option<T>>(keys: &[Key], f: F) -> T {
    keys.iter()
        .find_map(f)
        .expect("we failed to parse every key in the parsing step")
}

pub fn read_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Config> {
    let mut handle = std::fs::OpenOptions::new().read(true).open(&path).map_err(|_| {
        Error::NotFound(format!(
            "config file: {} not found",
            path.as_ref().to_path_buf().to_str().unwrap()
        ))
    })?;

    decode_config(&mut handle)
}

fn decode_config<R: std::io::Read>(handle: &mut R) -> Result<Config> {
    let mut buffer = String::default();
    handle
        .read_to_string(&mut buffer)
        .map_err(|_| Error::NonUtf8("config file is not valid utf8"))?;

    let mut lexer = lexer::Lexer::new(&buffer);
    let mut parser = parser::Parser::new(&buffer, &mut lexer);
    let config = parser.parse()?;
    Ok(config)
}
