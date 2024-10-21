mod common;
pub mod error;
mod import;
mod instructions;
mod syntax;

use common::expect;
pub use error::Result;
use error::{unexpected_eof, unexpected_token, PLUS_MSG};
use import::*;
use instructions::*;
use syntax::*;

use crate::ast::{Ast, Statement};
use crate::lexer::{Kind, Lexer, TransposeRef};

fn parse_instruction<S: AsRef<str>>(source: S, lexer: &mut Lexer, kind: Kind) -> Result<Statement> {
    match kind {
        Kind::Mov => parse_mov(source, lexer),
        Kind::Add => parse_add(source, lexer),
        Kind::Sub => parse_sub(source, lexer),
        Kind::Mul => parse_mul(source, lexer),
        Kind::Lsh => parse_lsh(source, lexer),
        Kind::Rsh => parse_rsh(source, lexer),
        Kind::And => parse_and(source, lexer),
        Kind::Or => parse_or(source, lexer),
        Kind::Xor => parse_xor(source, lexer),
        Kind::Inc => parse_inc(source, lexer),
        Kind::Dec => parse_dec(source, lexer),
        Kind::Not => parse_not(source, lexer),
        Kind::Jeq => parse_jeq(source, lexer),
        Kind::Jgt => parse_jgt(source, lexer),
        Kind::Jne => parse_jne(source, lexer),
        Kind::Jge => parse_jge(source, lexer),
        Kind::Jle => parse_jle(source, lexer),
        Kind::Jlt => parse_jlt(source, lexer),
        Kind::Psh => parse_psh(source, lexer),
        Kind::Pop => parse_pop(source, lexer),
        Kind::Call => parse_call(source, lexer),
        Kind::Ret => parse_ret(source, lexer),
        Kind::Hlt => parse_hlt(source, lexer),
        _ => unreachable!(),
    }
}

fn parse_exported_identifier<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    expect(Kind::Plus, lexer, source.as_ref(), "expected a `+` [PLUS]", PLUS_MSG)?;

    let Ok(Some(token)) = lexer.peek().transpose() else {
        let Err(err) = lexer.next().transpose() else {
            return unexpected_eof(source.as_ref(), "unterminated import statement");
        };
        return Err(err);
    };

    match token.kind {
        Kind::Ident => parse_label(source, lexer, true),
        Kind::Data8 => parse_data(source.as_ref(), lexer, DataSize::Byte, true),
        Kind::Data16 => parse_data(source.as_ref(), lexer, DataSize::Word, true),
        Kind::Const => parse_const(source.as_ref(), lexer, true),
        _ => unexpected_token(source.as_ref(), token),
    }
}

fn parse_statement<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    let Ok(Some(token)) = lexer.peek().transpose() else {
        let Err(err) = lexer.next().transpose() else {
            return unexpected_eof(source.as_ref(), "unterminated import statement");
        };
        return Err(err);
    };
    let kind = token.kind;
    match kind {
        Kind::Import => parse_import(source, lexer),
        Kind::Plus => parse_exported_identifier(source, lexer),
        Kind::Data8 => parse_data(source.as_ref(), lexer, DataSize::Byte, false),
        Kind::Data16 => parse_data(source.as_ref(), lexer, DataSize::Word, false),
        Kind::Const => parse_const(source, lexer, false),
        Kind::Ident => parse_label(source, lexer, false),
        k if k.is_instruction() => parse_instruction(source, lexer, kind),
        _ => unexpected_token(source.as_ref(), token),
    }
}

pub fn parse<S: AsRef<str>>(source: S) -> Result<Ast> {
    set_miette_hook();

    let source = source.as_ref();
    let mut lexer = Lexer::new(source);
    let mut statements = vec![];

    while !lexer.is_empty() {
        let statement = parse_statement(source, &mut lexer)?;
        statements.push(statement);
    }

    Ok(Ast { statements })
}

fn set_miette_hook() {
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .context_lines(10)
                .tab_width(2)
                .color(true)
                .break_words(true)
                .build(),
        )
    }))
    .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_label() {
        let input = "name:";
        let result = parse(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_exported_label() {
        let input = "+name:";
        let result = parse(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_private_constant() {
        let input = "const NAME = $0123";
        let result = parse(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_exported_constant() {
        let input = "+const NAME = &[$0123]";
        let result = parse(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_private_data8() {
        let input = "data8 NAME = { &[$0123], $1234 }";
        let result = parse(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_exported_data8() {
        let input = "+data8 NAME = { &[$0123], $1234, }";
        let result = parse(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_private_data16() {
        let input = "data16 NAME = { &[$0123], $1234 }";
        let result = parse(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_exported_data16() {
        let input = "+data16 NAME = { &[$0123], $1234, }";
        let result = parse(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }
}
