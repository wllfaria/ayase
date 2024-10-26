#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteOffset {
    start: usize,
    end: usize,
}

impl From<std::ops::Range<usize>> for ByteOffset {
    fn from(value: std::ops::Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<ByteOffset> for std::ops::Range<usize> {
    fn from(value: ByteOffset) -> Self {
        value.start..value.end
    }
}

impl From<ByteOffset> for miette::SourceSpan {
    fn from(value: ByteOffset) -> Self {
        (value.start..value.end).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
    pub offset: ByteOffset,
    pub kind: Kind,
}

impl Token {
    pub fn new(kind: Kind, offset: impl Into<ByteOffset>) -> Self {
        Self {
            kind,
            offset: offset.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Equal,
    Ident,
    String,
    Comma,
    LeftBracket,
    RightBracket,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Equal => write!(f, "EQUAL"),
            Kind::Ident => write!(f, "IDENT"),
            Kind::String => write!(f, "STRING"),
            Kind::Comma => write!(f, "COMMA"),
            Kind::LeftBracket => write!(f, "LEFT_BRACKET"),
            Kind::RightBracket => write!(f, "RIGHT_BRACKET"),
        }
    }
}

#[derive(Debug)]
pub struct Lexer<'lex> {
    source: &'lex str,
    full_source: &'lex str,
    pos: usize,
    peeked: Option<miette::Result<Token>>,
}

impl<'lex> Lexer<'lex> {
    pub fn new(source: &'lex str) -> Self {
        Self {
            source,
            full_source: source,
            pos: 0,
            peeked: None,
        }
    }

    pub fn peek(&mut self) -> Option<&miette::Result<Token>> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }

        self.peeked.as_ref()
    }

    fn advance(&mut self, amount: usize) {
        self.source = &self.source[amount..];
        self.pos += amount;
    }

    fn lex_ident(&mut self) -> Token {
        let start = self.pos;

        let end_of_ident = self
            .source
            .find(|ch| !matches!(ch, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'))
            .unwrap_or(self.source.len());
        self.advance(end_of_ident);
        Token::new(Kind::Ident, start..start + end_of_ident)
    }

    fn lex_string(&mut self) -> miette::Result<Token> {
        self.advance(1);
        let start = self.pos;
        let end_of_string = self.source.find(['"', '\n']).unwrap_or(self.source.len());

        self.advance(end_of_string);
        let next = self.source.chars().nth(0);
        match next {
            Some('\n') | None => Err(self.bail(
                "did you forget a closing \"",
                "unterminated string",
                start,
                end_of_string + 1,
            )),
            _ => {
                self.advance(1);
                Ok(Token::new(Kind::String, start..start + end_of_string))
            }
        }
    }

    fn bail(&self, help: &str, message: &str, start: usize, size: usize) -> miette::Error {
        miette::Error::from(
            miette::MietteDiagnostic::new(message)
                .with_labels(vec![miette::LabeledSpan::at(start..start + size, "this bit")])
                .with_help(help),
        )
        .with_source_code(self.full_source.to_string())
    }

    pub fn expect(&mut self, kind: Kind) -> miette::Result<Token> {
        let Some(token) = self.next().transpose()? else {
            return Err(self.bail(
                "[SYNTAX_ERROR]: unexpected end of file (EOF)",
                &format!("expected {kind}, found EOF",),
                self.full_source.len().saturating_sub(1),
                self.full_source.len(),
            ));
        };

        if token.kind != kind {
            return Err(self.bail(
                "[SYNTAX_ERROR]: unexpected token",
                &format!("expected {kind}, found {}", token.kind),
                token.offset.start,
                token.offset.start + token.offset.end,
            ));
        };

        Ok(token)
    }
}

impl<'lex> Iterator for Lexer<'lex> {
    type Item = miette::Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(peeked) = self.peeked.take() {
                return Some(peeked);
            }

            let mut chars = self.source.chars();
            let curr = chars.next()?;

            break match curr {
                ch if ch.is_whitespace() => {
                    self.advance(ch.len_utf8());
                    continue;
                }
                '=' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::Equal, self.pos..self.pos + 1)))
                }
                ',' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::Comma, self.pos..self.pos + 1)))
                }
                '[' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::LeftBracket, self.pos..self.pos + 1)))
                }
                ']' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::RightBracket, self.pos..self.pos + 1)))
                }
                '"' => Some(self.lex_string()),
                'a'..='z' | 'A'..='Z' | '_' => Some(Ok(self.lex_ident())),
                _ => Some(Err(self.bail(
                    &format!("unexpected token {curr}"),
                    "[SYNTAX_ERROR]: unexpected token",
                    self.pos,
                    self.pos + 1,
                ))),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_config() {
        let input = r#"
            code = "main.aya"
            sprites = "assets/spritesheet.bmp"
        "#;

        let tokens = Lexer::new(input).map(|t| t.unwrap()).collect::<Vec<_>>();
        insta::assert_debug_snapshot!(tokens);
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

        let tokens = Lexer::new(input).map(|t| t.unwrap()).collect::<Vec<_>>();
        insta::assert_debug_snapshot!(tokens);
    }

    #[test]
    #[should_panic]
    fn test_syntax_error() {
        let input = r#"
            code = "main.aya"
            sprites = {
                "assets/01.bmp",
                "assets/02.bmp",
                "assets/03.bmp",
            }
        "#;

        _ = Lexer::new(input).map(|t| t.unwrap()).collect::<Vec<_>>();
    }
}
