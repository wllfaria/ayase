use crate::config::lexer::{Kind, Lexer};
use crate::config::Config;

#[derive(Debug)]
pub struct Parser<'par> {
    source: &'par str,
    lexer: &'par mut Lexer<'par>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Key {
    Code(crate::config::lexer::ByteOffset),
    Sprites(Vec<crate::config::lexer::ByteOffset>),
}

impl<'par> Parser<'par> {
    pub fn new(source: &'par str, lexer: &'par mut Lexer<'par>) -> Self {
        Self { source, lexer }
    }

    pub fn parse(&mut self) -> miette::Result<Config> {
        let mut keys = vec![];
        while self.lexer.peek().is_some() {
            keys.push(parse_key(self.source, self.lexer)?);
        }

        Ok(Config::from_keys(self.source, keys))
    }
}

fn parse_key<'par>(source: &'par str, lexer: &mut Lexer<'par>) -> miette::Result<Key> {
    let Some(token) = lexer.next().transpose()? else {
        return Err(bail(source, "[SYNTAX_ERROR]: unexpected end of file (EOF)", "lol"));
    };

    if token.kind != Kind::Ident {
        return Err(bail(
            source,
            "[SYNTAX_ERROR]: unexpected token",
            &format!("expected IDENT, found {}", token.kind),
        ));
    };

    let ident = &source[std::ops::Range::<usize>::from(token.offset)];

    let key = match ident {
        "code" => parse_code_key(lexer)?,
        "sprites" => parse_sprites_key(source, lexer)?,
        _ => {
            return Err(bail(
                source,
                "[SYNTAX_ERROR]: unexpected key",
                &format!("the key {ident} is not a valid config key"),
            ))
        }
    };

    Ok(key)
}

fn parse_code_key(lexer: &mut Lexer<'_>) -> miette::Result<Key> {
    lexer.expect(Kind::Equal)?;
    let token = lexer.expect(Kind::String)?;
    Ok(Key::Code(token.offset))
}

fn parse_sprites_key<'par>(source: &'par str, lexer: &mut Lexer<'par>) -> miette::Result<Key> {
    lexer.expect(Kind::Equal)?;

    let Some(token) = lexer.next().transpose()? else {
        return Err(bail(
            source,
            "[SYNTAX_ERROR]: unexpected end of file (EOF)",
            "expected value for sprite path",
        ));
    };

    let key = match token.kind {
        Kind::LeftBracket => parse_sprites_array(source, lexer)?,
        Kind::String => Key::Sprites(vec![token.offset]),
        _ => {
            return Err(bail(
                source,
                "[SYNTAX_ERROR]: unexpected token",
                "expected value for sprite path",
            ))
        }
    };

    Ok(key)
}

fn parse_sprites_array<'par>(source: &'par str, lexer: &mut Lexer<'par>) -> miette::Result<Key> {
    let mut offsets = vec![];

    loop {
        let Some(token) = lexer.next().transpose()? else {
            return Err(bail(
                source,
                "[SYNTAX_ERROR]: unexpected end of file (EOF)",
                "expected value for sprite path",
            ));
        };

        let offset = match token.kind {
            Kind::RightBracket => break,
            Kind::String => token.offset,
            _ => {
                return Err(bail(
                    source,
                    "[SYNTAX_ERROR]: unexpected token",
                    "sprite paths must be strings",
                ))
            }
        };

        offsets.push(offset)
    }

    Ok(Key::Sprites(offsets))
}

fn bail<S: AsRef<str>>(source: &str, message: S, help: S) -> miette::Error {
    miette::Error::from(
        miette::MietteDiagnostic::new(message.as_ref())
            .with_labels(vec![miette::LabeledSpan::at(
                source.len() - 1..source.len(),
                "this bit",
            )])
            .with_help(help.as_ref()),
    )
    .with_source_code(source.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sut(input: &str) -> Config {
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(input, &mut lexer);
        parser.parse().unwrap()
    }

    #[test]
    fn test_simple_config() {
        let input = r#"
            code = "main.aya"
            sprites = "assets/spritesheet.bmp"
        "#;
        let expected = Config {
            code: String::from("main.aya"),
            sprites: vec![String::from("assets/spritesheet.bmp")],
        };

        let config = make_sut(input);
        assert_eq!(config, expected);
    }

    #[test]
    fn test_sprite_array() {
        let input = r#"
            code = "main.aya"
            sprites = [
                "assets/01.bmp",
                "assets/02.bmp",
                "assets/03.bmp",
            ]
        "#;
        let expected = Config {
            code: String::from("main.aya"),
            sprites: vec![
                String::from("assets/01.bmp"),
                String::from("assets/02.bmp"),
                String::from("assets/03.bmp"),
            ],
        };

        let config = make_sut(input);

        assert_eq!(config, expected);
    }
}
