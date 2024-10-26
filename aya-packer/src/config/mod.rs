mod lexer;
mod parser;

use parser::Key;

use crate::error::{Error, Result};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Config {
    pub code: String,
    pub sprites: Vec<String>,
}

impl Config {
    pub(crate) fn from_keys(source: &str, keys: Vec<Key>) -> Self {
        let code = keys
            .iter()
            .find_map(|key| {
                let Key::Code(offset) = key else {
                    return None;
                };
                Some(*offset)
            })
            .unwrap();

        let code = source[std::ops::Range::<usize>::from(code)].to_string();

        let sprites = keys
            .iter()
            .find_map(|key| {
                let Key::Sprites(offsets) = key else {
                    return None;
                };
                Some(offsets.clone())
            })
            .unwrap();

        let sprites = sprites
            .into_iter()
            .map(|offset| source[std::ops::Range::<usize>::from(offset)].to_string())
            .collect::<Vec<_>>();

        Self { code, sprites }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            code: String::from("main.aya"),
            sprites: vec![String::from("asstes/sprite.bmp")],
        }
    }
}

pub fn default() -> Config {
    Default::default()
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
