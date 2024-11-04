use miette::Result;

use super::ast::{Operator, Statement};
use super::common::{expect, parse_hex_lit, parse_register, parse_variable, peek};
use super::error::{HEX_LIT_HELP, HEX_LIT_MSG};
use crate::lexer::{Kind, Lexer};
use crate::utils::unexpected_token;

mod precedences {
    use miette::Result;

    use crate::parser::ast::Operator;

    pub const BASE: u8 = 0;
    pub const ADD: u8 = 1;
    pub const MUL: u8 = 2;

    pub fn from_operator(operator: Operator) -> Result<u8> {
        match operator {
            Operator::Add => Ok(ADD),
            Operator::Sub => Ok(ADD),
            Operator::Mul => Ok(MUL),
        }
    }
}

pub fn parse_literal_expr<S: AsRef<str>>(source: S, lexer: &mut Lexer, help: S, message: S) -> Result<Statement> {
    expect(Kind::LBracket, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;
    let value = parse_expr(source.as_ref(), lexer, precedences::BASE)?;
    expect(Kind::RBracket, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;
    Ok(value)
}

pub fn parse_address_expr<S: AsRef<str>>(source: S, lexer: &mut Lexer, help: S, message: S) -> Result<Statement> {
    expect(Kind::Ampersand, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;
    expect(Kind::LBracket, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;

    let value = parse_expr(source.as_ref(), lexer, precedences::BASE)?;

    expect(Kind::RBracket, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;
    Ok(Statement::Address(Box::new(value)))
}

fn parse_expr<S: AsRef<str>>(source: S, lexer: &mut Lexer, precedence: u8) -> Result<Statement> {
    let token = peek(source.as_ref(), lexer)?;
    let mut lhs = match token.kind {
        Kind::LParen => {
            lexer.next().transpose()?;
            let value = parse_expr(source.as_ref(), lexer, precedences::BASE)?;
            expect(
                Kind::RParen,
                lexer,
                source.as_ref(),
                "you likely forgot a closing parenthesis `)` [RIGHT_PAREN]",
                "[SYNTAX_ERROR]: unterminated expression group",
            )?;
            value
        }
        Kind::HexNumber => Statement::HexLiteral(parse_hex_lit(source.as_ref(), lexer, HEX_LIT_HELP, HEX_LIT_MSG)?),
        Kind::Ident => Statement::Register(parse_register(source.as_ref(), lexer)?),
        Kind::Bang => Statement::Var(parse_variable(
            source.as_ref(),
            lexer,
            "variable name must be a valid identifier",
            "[SYNTAX_ERROR]: invalid variable name",
        )?),
        _ => unexpected_token(source.as_ref(), &token)?,
    };

    loop {
        let token = peek(source.as_ref(), lexer)?;
        match token.kind {
            Kind::RParen => break,
            Kind::RBracket => break,
            kind if !kind.is_operator() => unexpected_token(source.as_ref(), &token)?,
            _ => {}
        }

        let operator = peek(source.as_ref(), lexer)?;
        let operator = Operator::try_from(operator)?;
        let operator_precedence = precedences::from_operator(operator)?;

        if operator_precedence < precedence {
            break;
        }

        lexer.next().transpose()?;

        let rhs = parse_expr(source.as_ref(), lexer, operator_precedence)?;
        lhs = Statement::BinaryOp {
            lhs: Box::new(lhs),
            operator,
            rhs: Box::new(rhs),
        }
    }

    Ok(lhs)
}
