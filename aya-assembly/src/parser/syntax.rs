use super::Result;
use crate::lexer::{Kind, Lexer, TransposeRef};
use crate::parser::ast::Statement;
use crate::parser::common::{expect, expect_fail, parse_hex_lit, parse_identifier, parse_variable};
use crate::parser::error::{
    unexpected_eof, unexpected_token, ADDRESS_HELP, ADDRESS_MSG, COMMA_MSG, HEX_LIT_HELP, HEX_LIT_MSG, IDENT_MSG,
    LBRACE_MSG, RBRACE_MSG,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum DataSize {
    Byte,
    Word,
}

impl From<DataSize> for u8 {
    fn from(size: DataSize) -> Self {
        match size {
            DataSize::Byte => 8,
            DataSize::Word => 16,
        }
    }
}

pub fn parse_address_ident<S: AsRef<str>>(source: S, lexer: &mut Lexer, help: S, message: S) -> Result<Statement> {
    expect(Kind::Ampersand, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;
    expect(Kind::LBracket, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;

    let Ok(Some(token)) = lexer.peek().transpose() else {
        let Err(err) = lexer.next().transpose() else {
            return unexpected_eof(source.as_ref(), "unterminated import statement");
        };
        return Err(err);
    };

    let value = match token.kind {
        Kind::HexNumber => Statement::Address(Box::new(Statement::HexLiteral(parse_hex_lit(
            source.as_ref(),
            lexer,
            help.as_ref(),
            message.as_ref(),
        )?))),
        Kind::Bang => Statement::Var(parse_variable(source.as_ref(), lexer, help.as_ref(), message.as_ref())?),
        Kind::Ident => Statement::Register(parse_identifier(
            source.as_ref(),
            lexer,
            help.as_ref(),
            message.as_ref(),
        )?),
        _ => return unexpected_token(source.as_ref(), token),
    };

    expect(Kind::RBracket, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;
    Ok(value)
}

pub fn parse_simple_address<S: AsRef<str>>(source: S, lexer: &mut Lexer, help: S, message: S) -> Result<Statement> {
    expect(Kind::Ampersand, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;
    expect(Kind::LBracket, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;

    let Ok(Some(token)) = lexer.peek().transpose() else {
        let Err(err) = lexer.next().transpose() else {
            return unexpected_eof(source.as_ref(), "unterminated import statement");
        };
        return Err(err);
    };

    let value = match token.kind {
        Kind::HexNumber => Statement::Address(Box::new(Statement::HexLiteral(parse_hex_lit(
            source.as_ref(),
            lexer,
            help.as_ref(),
            message.as_ref(),
        )?))),
        Kind::Bang => Statement::Var(parse_variable(source.as_ref(), lexer, help.as_ref(), message.as_ref())?),
        _ => return unexpected_token(source.as_ref(), token),
    };

    expect(Kind::RBracket, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;
    Ok(value)
}

pub fn parse_label<S: AsRef<str>>(source: S, lexer: &mut Lexer, exported: bool) -> Result<Statement> {
    let name = parse_identifier(
        source.as_ref(),
        lexer,
        "label name must be a valid identifier",
        IDENT_MSG,
    )?;
    expect_fail(Kind::Colon, lexer, source.as_ref())?;
    Ok(Statement::Label { name, exported })
}

pub fn parse_const<S: AsRef<str>>(source: S, lexer: &mut Lexer, exported: bool) -> Result<Statement> {
    expect_fail(Kind::Const, lexer, source.as_ref())?;

    let name = parse_identifier(
        source.as_ref(),
        lexer,
        "constant name must be a valid identifier",
        IDENT_MSG,
    )?;

    expect_fail(Kind::Equal, lexer, source.as_ref())?;

    let Ok(Some(next)) = lexer.peek().transpose() else {
        let Err(err) = lexer.next().transpose() else {
            return unexpected_eof(source.as_ref(), "unterminated import statement");
        };
        return Err(err);
    };

    let value = match next.kind {
        Kind::Ampersand => parse_simple_address(source.as_ref(), lexer, ADDRESS_HELP, ADDRESS_MSG)?,
        Kind::HexNumber => Statement::HexLiteral(parse_hex_lit(source.as_ref(), lexer, HEX_LIT_HELP, HEX_LIT_MSG)?),
        _ => return unexpected_token(source.as_ref(), next),
    };

    Ok(Statement::Const {
        name,
        exported,
        value: Box::new(value),
    })
}

pub fn parse_data<S: AsRef<str>>(source: S, lexer: &mut Lexer, size: DataSize, exported: bool) -> Result<Statement> {
    match size {
        DataSize::Byte => expect_fail(Kind::Data8, lexer, source.as_ref())?,
        DataSize::Word => expect_fail(Kind::Data16, lexer, source.as_ref())?,
    };

    let name = parse_identifier(
        source.as_ref(),
        lexer,
        "data name must be a valid identifier",
        IDENT_MSG,
    )?;

    expect_fail(Kind::Equal, lexer, source.as_ref())?;

    expect(
        Kind::LBrace,
        lexer,
        source.as_ref(),
        "data variables must be surrounded by curly braces",
        LBRACE_MSG,
    )?;

    let values = parse_data_values(source.as_ref(), lexer)?;

    expect(
        Kind::RBrace,
        lexer,
        source.as_ref(),
        "unclosed data declaration block. you most likely forgot a `}` [RIGHT_CURLY]",
        RBRACE_MSG,
    )?;

    Ok(Statement::Data {
        name,
        size: size.into(),
        exported,
        values,
    })
}

fn parse_data_values<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Vec<Statement>> {
    let mut values = vec![];

    loop {
        let Ok(Some(next)) = lexer.peek().transpose() else {
            let Err(err) = lexer.next().transpose() else {
                return unexpected_eof(source.as_ref(), "unterminated import statement");
            };
            return Err(err);
        };

        let value = match next.kind {
            Kind::RBrace => break,
            Kind::Ampersand => parse_simple_address(source.as_ref(), lexer, ADDRESS_HELP, ADDRESS_MSG)?,
            Kind::HexNumber => Statement::HexLiteral(parse_hex_lit(source.as_ref(), lexer, HEX_LIT_HELP, HEX_LIT_MSG)?),
            _ => return unexpected_token(source.as_ref(), next),
        };

        let Ok(Some(next)) = lexer.peek().transpose() else {
            let Err(err) = lexer.next().transpose() else {
                return unexpected_eof(source.as_ref(), "unterminated import statement");
            };
            return Err(err);
        };

        match next.kind {
            Kind::RBrace => {}
            _ => {
                _ = expect(
                    Kind::Comma,
                    lexer,
                    source.as_ref(),
                    "import variables must be separated by a comma",
                    COMMA_MSG,
                )?
            }
        }

        values.push(value);
    }

    Ok(values)
}
