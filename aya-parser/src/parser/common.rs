use std::ops::Range;

use super::error::{bail, unexpected_eof, unexpected_token};
use super::Result;
use crate::ast::ByteOffset;
use crate::lexer::{Kind, Lexer, Token, TransposeRef};

pub fn expect<S: AsRef<str>>(
    expected: Kind,
    lexer: &mut Lexer,
    source: S,
    help: S,
    message: S,
) -> miette::Result<ByteOffset> {
    let current = lexer.next().transpose()?;
    let Some(current) = current else {
        unreachable!();
    };

    if current.kind != expected {
        return Err(bail(source.as_ref(), help.as_ref(), message.as_ref(), current.offset()));
    }

    Ok(current.offset())
}

pub fn expect_fail<S: AsRef<str>>(expected: Kind, lexer: &mut Lexer, source: S) -> miette::Result<bool> {
    let Ok(Some(next)) = lexer.peek().transpose() else {
        let Err(err) = lexer.next().transpose() else {
            return unexpected_eof(source.as_ref(), "unterminated import statement");
        };
        return Err(err);
    };

    if next.kind != expected {
        return unexpected_token(source.as_ref(), next);
    }

    lexer.next().transpose()?;
    Ok(false)
}

pub fn parse_identifier<S: AsRef<str>>(source: S, lexer: &mut Lexer, help: S, message: S) -> Result<ByteOffset> {
    expect(Kind::Ident, lexer, source.as_ref(), help.as_ref(), message.as_ref())
}

pub fn parse_register<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<ByteOffset> {
    let offset = parse_identifier(source.as_ref(), lexer, "", "")?;
    let name = &source.as_ref()[Range::<usize>::from(offset)];
    match name {
        "acc" | "ip" | "r1" | "r2" | "r3" | "r4" | "r5" | "r6" | "r7" | "r8" | "sp" | "fp" => Ok(offset),
        _ => unexpected_token(source.as_ref(), &Token::from_ident(name, offset.start, offset.end)),
    }
}

pub fn parse_hex_lit<S: AsRef<str>>(source: S, lexer: &mut Lexer, help: S, message: S) -> Result<ByteOffset> {
    expect(Kind::HexNumber, lexer, source.as_ref(), help.as_ref(), message.as_ref())
}

pub fn parse_string<S: AsRef<str>>(source: S, lexer: &mut Lexer, help: S, message: S) -> Result<ByteOffset> {
    expect(Kind::String, lexer, source.as_ref(), help.as_ref(), message.as_ref())
}

pub fn parse_variable<S: AsRef<str>>(source: S, lexer: &mut Lexer, help: S, message: S) -> Result<ByteOffset> {
    expect(Kind::Bang, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;
    expect(Kind::Ident, lexer, source.as_ref(), help.as_ref(), message.as_ref())
}

pub fn parse_keyword<S: AsRef<str>>(source: S, lexer: &mut Lexer, expected: Kind) -> Result<ByteOffset> {
    let next = lexer.next().transpose()?;
    let Some(next) = next else {
        return unexpected_eof(source.as_ref(), "invalid eof");
    };

    if next.kind != expected {
        return unexpected_token(source.as_ref(), &next);
    }

    Ok(next.offset())
}
