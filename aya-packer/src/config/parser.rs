use crate::config::lexer::{ByteOffset, Kind, Lexer, TransposeRef};
use crate::config::Config;

#[derive(Debug)]
pub struct Parser<'par> {
    source: &'par str,
    lexer: &'par mut Lexer<'par>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Key {
    Code(ByteOffset),
    Sprites(Vec<ByteOffset>),
    Name(ByteOffset),
    Output(ByteOffset),
    Expand(ByteOffset),
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::Code(_) => write!(f, "code"),
            Key::Sprites(_) => write!(f, "sprites"),
            Key::Name(_) => write!(f, "name"),
            Key::Output(_) => write!(f, "output"),
            Key::Expand(_) => write!(f, "expand"),
        }
    }
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
        return Err(bail(
            source,
            "[SYNTAX_ERROR]: unexpected end of file (EOF)",
            "expected valid key for config",
            source.len().saturating_sub(1)..source.len(),
        ));
    };

    if token.kind != Kind::Ident {
        return Err(bail(
            source,
            "[SYNTAX_ERROR]: unexpected token",
            &format!("expected IDENT, found {}", token.kind),
            token.offset,
        ));
    };

    let ident = &source[std::ops::Range::<usize>::from(token.offset)];

    let key = match ident {
        "sprites" => parse_sprites_key(source, lexer)?,
        "code" => parse_code_key(lexer)?,
        "output" => parse_output_key(lexer)?,
        "name" => parse_name_key(lexer)?,
        "expand" => parse_expand_key(lexer)?,
        _ => {
            return Err(bail(
                source,
                "[SYNTAX_ERROR]: unexpected key",
                &format!("the key '{ident}' is not a valid config key"),
                token.offset,
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

fn parse_name_key(lexer: &mut Lexer<'_>) -> miette::Result<Key> {
    lexer.expect(Kind::Equal)?;
    let token = lexer.expect(Kind::String)?;
    Ok(Key::Name(token.offset))
}

fn parse_output_key(lexer: &mut Lexer<'_>) -> miette::Result<Key> {
    lexer.expect(Kind::Equal)?;
    let token = lexer.expect(Kind::String)?;
    Ok(Key::Output(token.offset))
}

fn parse_expand_key(lexer: &mut Lexer<'_>) -> miette::Result<Key> {
    lexer.expect(Kind::Equal)?;
    let token = lexer.expect(Kind::Bool)?;
    Ok(Key::Expand(token.offset))
}

fn parse_sprites_key<'par>(source: &'par str, lexer: &mut Lexer<'par>) -> miette::Result<Key> {
    lexer.expect(Kind::Equal)?;

    let Some(token) = lexer.next().transpose()? else {
        return Err(bail(
            source,
            "[SYNTAX_ERROR]: unexpected end of file (EOF)",
            "expected value for sprite path",
            source.len().saturating_sub(1)..source.len(),
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
                token.offset,
            ))
        }
    };

    Ok(key)
}

fn parse_sprites_array<'par>(source: &'par str, lexer: &mut Lexer<'par>) -> miette::Result<Key> {
    let mut offsets = vec![];

    loop {
        let Ok(Some(token)) = lexer.peek().transpose() else {
            let Err(err) = lexer.next().transpose() else {
                return Err(bail(
                    source,
                    "[SYNTAX_ERROR]: unexpected end of file (EOF)",
                    "expected value for sprite path",
                    source.len().saturating_sub(1)..source.len(),
                ));
            };
            return Err(err);
        };

        let offset = match token.kind {
            Kind::RightBracket => break,
            Kind::String => parse_string(lexer)?,
            _ => {
                return Err(bail(
                    source,
                    "[SYNTAX_ERROR]: unexpected token",
                    "sprite paths must be strings",
                    token.offset,
                ));
            }
        };

        let Ok(Some(next)) = lexer.peek().transpose() else {
            let Err(err) = lexer.next().transpose() else {
                return Err(bail(
                    source,
                    "[SYNTAX_ERROR]: unexpected end of file (EOF)",
                    "expected value for sprite path",
                    source.len().saturating_sub(1)..source.len(),
                ));
            };
            return Err(err);
        };

        match next.kind {
            Kind::RightBracket => {}
            _ => _ = lexer.expect(Kind::Comma)?,
        }

        offsets.push(offset)
    }

    lexer.expect(Kind::RightBracket)?;

    Ok(Key::Sprites(offsets))
}

fn parse_string(lexer: &mut Lexer) -> miette::Result<ByteOffset> {
    let token = lexer.expect(Kind::String)?;
    Ok(token.offset)
}

fn bail<S: AsRef<str>>(source: &str, message: S, help: S, span: impl Into<miette::SourceSpan>) -> miette::Error {
    miette::Error::from(
        miette::MietteDiagnostic::new(message.as_ref())
            .with_labels(vec![miette::LabeledSpan::at(span, "this bit")])
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
            name = "hello"
            code = "main.aya"
            output = "my_game.out"
            sprites = "assets/spritesheet.bmp"
        "#;
        let expected = Config {
            name: String::from("hello"),
            output: String::from("my_game.out"),
            code: String::from("main.aya"),
            sprites: vec![String::from("assets/spritesheet.bmp")],
            expand: false,
        };

        let config = make_sut(input);
        assert_eq!(config, expected);
    }

    #[test]
    fn test_sprite_array() {
        let input = r#"
            code = "main.aya"
            name = "hello"
            output = "my_game.out"
            sprites = [
                "assets/01.bmp",
                "assets/02.bmp",
                "assets/03.bmp",
            ]
        "#;
        let expected = Config {
            name: String::from("hello"),
            code: String::from("main.aya"),
            output: String::from("my_game.out"),
            sprites: vec![
                String::from("assets/01.bmp"),
                String::from("assets/02.bmp"),
                String::from("assets/03.bmp"),
            ],
            expand: false,
        };

        let config = make_sut(input);

        assert_eq!(config, expected);
    }

    #[test]
    #[should_panic]
    fn test_syntax_error() {
        let input = r#"
            code = "main.aya"
            name = "hello"
            output = "my_game.out"
            sprites = [
                "assets/01.bmp",
                "assets/02.bmp"
                "assets/03.bmp",
            ]
        "#;

        make_sut(input);
    }

    #[test]
    #[should_panic]
    fn test_invalid_key() {
        let input = r#"
            code = "main.aya"
            output = "my_game.out"
            sprites = [
                "assets/01.bmp",
                "assets/02.bmp",
                "assets/03.bmp",
            ]
            name = "my game"
            invalid = "key"
        "#;

        make_sut(input);
    }
}
