mod token;

pub use token::{Kind, Token};

use crate::parser::error::{UNTERMINATED_STRING_HELP, UNTERMINATED_STRING_MSG};
use crate::utils::bail;
pub type Result<T> = std::result::Result<T, miette::Error>;

pub trait TransposeRef<'a, T, E> {
    fn transpose(self) -> std::result::Result<Option<&'a T>, &'a E>;
}

impl<'lex> TransposeRef<'lex, Token, miette::Error> for Option<&'lex Result<Token>> {
    fn transpose(self) -> std::result::Result<Option<&'lex Token>, &'lex miette::Error> {
        match self {
            Some(result) => match result {
                Ok(token) => Ok(Some(token)),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
    }
}

pub struct Lexer<'lex> {
    full_source: &'lex str,
    source: &'lex str,
    pos: usize,
    peeked: Option<Result<Token>>,
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

    pub fn peek(&mut self) -> Option<&Result<Token>> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }

        self.peeked.as_ref()
    }

    pub fn is_empty(&mut self) -> bool {
        match self.peek() {
            None => true,
            Some(Ok(token)) if matches!(token.kind, Kind::Eof) => true,
            _ => false,
        }
    }

    fn advance(&mut self, amount: usize) {
        self.source = &self.source[amount..];
        self.pos += amount;
    }

    fn lex_identifier(&mut self) -> Token {
        let start = self.pos;

        let end_of_ident = self
            .source
            .find(|ch| !matches!(ch, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'))
            .unwrap_or(self.source.len());
        let ident = &self.source[..end_of_ident];
        self.advance(end_of_ident);
        Token::from_ident(ident, start, start + end_of_ident)
    }

    fn lex_hex_number(&mut self) -> Token {
        let start = self.pos;
        let end_of_number = self
            .source
            .find(|ch: char| !ch.is_ascii_hexdigit())
            .unwrap_or(self.source.len());
        self.advance(end_of_number);
        Token::new(Kind::HexNumber, start..start + end_of_number)
    }

    fn lex_string(&mut self) -> miette::Result<Token> {
        self.advance(1);
        let start = self.pos;
        let end_of_string = self.source.find(['"', '\n']).unwrap_or(self.source.len());

        self.advance(end_of_string);
        let next = self.source.chars().nth(0);
        match next {
            Some('\n') | None => Err(bail(
                self.full_source,
                UNTERMINATED_STRING_HELP,
                UNTERMINATED_STRING_MSG,
                start..end_of_string + 1,
            )),
            _ => {
                self.advance(1);
                Ok(Token::new(Kind::String, start..start + end_of_string))
            }
        }
    }
}

impl<'lex> Iterator for Lexer<'lex> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(peeked) = self.peeked.take() {
                return Some(peeked);
            }

            let mut chars = self.source.chars().peekable();

            let ch = chars.next()?;

            break match ch {
                _ if ch.is_whitespace() => {
                    self.advance(ch.len_utf8());
                    continue;
                }
                '+' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::Plus, self.pos - 1..self.pos)))
                }
                '-' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::Minus, self.pos - 1..self.pos)))
                }
                '*' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::Star, self.pos - 1..self.pos)))
                }
                '!' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::Bang, self.pos - 1..self.pos)))
                }
                '&' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::Ampersand, self.pos - 1..self.pos)))
                }
                '[' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::LBracket, self.pos - 1..self.pos)))
                }
                ']' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::RBracket, self.pos - 1..self.pos)))
                }
                '(' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::LParen, self.pos - 1..self.pos)))
                }
                ')' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::RParen, self.pos - 1..self.pos)))
                }
                '{' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::LBrace, self.pos - 1..self.pos)))
                }
                '}' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::RBrace, self.pos - 1..self.pos)))
                }
                ':' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::Colon, self.pos - 1..self.pos)))
                }
                '$' => {
                    self.advance(1);
                    Some(Ok(self.lex_hex_number()))
                }
                '=' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::Equal, self.pos - 1..self.pos)))
                }
                ',' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::Comma, self.pos - 1..self.pos)))
                }
                ';' => {
                    let eol = self.source.find('\n').unwrap_or(self.source.len());
                    self.advance(eol);
                    continue;
                }
                '.' => {
                    self.advance(1);
                    Some(Ok(Token::new(Kind::Dot, self.pos - 1..self.pos)))
                }
                '"' => Some(self.lex_string()),
                'a'..='z' | 'A'..='Z' | '_' => Some(Ok(self.lex_identifier())),
                _ => Some(Ok(Token::new(Kind::Eof, self.pos..self.pos + 1))),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexing_language_spec() {
        let input = r#"
; Move instructions
mov r1,         $3000       ; mov literal into register                     (MovLitReg)
mov r1,         r2          ; mov register into register                    (MovRegReg)
mov &[$c0d3],   r3          ; mov register into memory                      (MovRegMem)
mov r1,         &[$3000]    ; mov memory into register                      (MovMemReg)
mov &[$3000],   $abcd       ; mov literal into memory                       (MovLitMem)
mov r1,         &[r2]       ; mov register pointer into register            (MovRegPtrReg)

; Math instructions
add r1,         r2          ; add register into register                    (AddRegReg)
add r1,         $0010       ; add literal into register                     (AddLitReg)
sub r1,         r2          ; sub register from register                    (SubRegReg)
sub r1,         $0010       ; sub literal from register                     (SubLitReg)
mul r1,         r2          ; multiply register with register               (MulRegReg)
mul r1,         $0010       ; multiply register with literal                (MulLitReg)
inc r1                      ; increment register                            (IncReg)
dec r1                      ; decrement register                            (DecReg)

; Binary instructions
lsh r1,         r2          ; left shift register with register             (LsfRegReg)
lsh r1,         $0010       ; left shift register with literal              (LsfLitReg)
rsh r1,         r2          ; right shift register with register            (RsfRegReg)
rsh r1,         $0010       ; right shift register with literal             (RsfLitReg)
and r1,         r2          ; and (&) register into register                (AndRegReg)
and r1,         $0010       ; and (&) literal into register                 (AndLitReg)
or  r1,         r2          ; or  (|) register into register                (OrRegReg)
or  r1,         $0010       ; or  (|) literal into register                 (OrLitReg)
xor r1,         r2          ; xor (^) register into register                (XorRegReg)
xor r1,         $0010       ; xor (^) literal into register                 (XorLitReg)
not r1                      ; not (~) register                              (Not)

; Memory instructions
psh r1                      ; push register into stack                      (PushReg)
psh $0010                   ; push literal into stack                       (PushLit)
pop r1                      ; pop from the stack into register              (Pop)
call &[$0100]               ; call subroutine on address                    (Call)
ret                         ; return from subroutine                        (Ret)

; Jump instructions
jeq &[$0000],   r2          ; jumps if register is equal to ret             (JeqReg)
jeq &[$0000],   $0000       ; jumps if literal is equal to ret              (JeqLit)
jgt &[$0000],   r2          ; jumps if register is greater than ret         (JgtReg)
jgt &[$0000],   $0000       ; jumps if literal is greater than ret          (JgtLit)
jne &[$0000],   r2          ; jumps if register is not equal to ret         (JneReg)
jne &[$0000],   $0000       ; jumps if literal is not equal to ret          (JneLit)
jge &[$0000],   r2          ; jumps if register is greater or equal to ret  (JgeReg)
jge &[$0000],   $0000       ; jumps if literal is greater or equal to ret   (JgeLit)
jle &[$0000],   r2          ; jumps if register is lesser or equal to ret   (JleReg)
jle &[$0000],   $0000       ; jumps if literal is lesser or equal to ret    (JleLit)
jlt &[$0000],   r2          ; jumps if register is lesser than ret          (JltReg)
jlt &[$0000],   $0000       ; jumps if literal is lesser than ret           (JltLit)
hlt                         ; halts the virtual machine                     (Halt)

; Module system syntax
import "./path.aya" ModuleName &[abcd] {
    variable1: !var,
    variable2: $0000,
    variable3: &[$0000],
    variable4: [OtherModule.variable],
}
"#;

        let lexer = Lexer::new(input);
        let tokens = lexer.into_iter().collect::<Vec<_>>();
        let tokens = tokens.into_iter().map(|tok| tok.unwrap()).collect::<Vec<_>>();
        insta::assert_debug_snapshot!(tokens);
    }
}
