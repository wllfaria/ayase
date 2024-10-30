use crate::lexer::{Kind, Lexer, Token, TransposeRef};
use crate::parser::ast::{ByteOffset, Instruction, Operator, Statement};
use crate::parser::common::{expect, parse_hex_lit, parse_keyword, parse_register, parse_variable};
use crate::parser::error::{
    unexpected_eof, unexpected_token, ADDRESS_HELP, ADDRESS_MSG, COMMA_MSG, HEX_LIT_HELP, HEX_LIT_MSG,
};
use crate::parser::syntax::{parse_address_ident, parse_simple_address};
use crate::parser::Result;

pub fn peek<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Token> {
    let Ok(Some(token)) = lexer.peek().transpose() else {
        let Err(err) = lexer.next().transpose() else {
            return unexpected_eof(source.as_ref(), "unterminated mov instruction");
        };
        return Err(err);
    };
    Ok(*token)
}

pub fn parse_mov<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    parse_keyword(source.as_ref(), lexer, Kind::Mov)?;

    let lhs_token = peek(source.as_ref(), lexer)?;

    let lhs = match lhs_token.kind {
        Kind::Ident => Statement::Register(parse_register(source.as_ref(), lexer)?),
        Kind::Ampersand => parse_address_expr(source.as_ref(), lexer, ADDRESS_HELP, ADDRESS_MSG)?,
        _ => return unexpected_token(source.as_ref(), &lhs_token),
    };

    expect(
        Kind::Comma,
        lexer,
        source.as_ref(),
        "missing a comma after left side of instruction",
        COMMA_MSG,
    )?;

    let rhs_token = peek(source.as_ref(), lexer)?;
    let rhs = match rhs_token.kind {
        Kind::Ident => Statement::Register(parse_register(source.as_ref(), lexer)?),
        Kind::Bang => Statement::Var(parse_variable(source.as_ref(), lexer, "", "")?),
        Kind::HexNumber => Statement::HexLiteral(parse_hex_lit(source.as_ref(), lexer, HEX_LIT_HELP, HEX_LIT_MSG)?),
        Kind::Ampersand => parse_simple_address(source.as_ref(), lexer, ADDRESS_HELP, ADDRESS_MSG)?,
        _ => return unexpected_token(source.as_ref(), &rhs_token),
    };

    match (lhs_token.kind, rhs_token.kind) {
        (Kind::Ident, Kind::Ident) => Ok(Instruction::MovRegReg(lhs, rhs).into()),
        (Kind::Ident, Kind::Bang) => Ok(Instruction::MovLitReg(lhs, rhs).into()),
        (Kind::Ident, Kind::HexNumber) => Ok(Instruction::MovLitReg(lhs, rhs).into()),
        (Kind::Ident, Kind::Ampersand) => Ok(Instruction::MovMemReg(lhs, rhs).into()),
        (Kind::Ampersand, _) if matches!(lhs, Statement::Register(_)) => Ok(Instruction::MovRegPtrReg(lhs, rhs).into()),
        (Kind::Ampersand, Kind::Ident) => Ok(Instruction::MovRegMem(lhs, rhs).into()),
        (Kind::Ampersand, Kind::Bang) => Ok(Instruction::MovLitMem(lhs, rhs).into()),
        (Kind::Ampersand, Kind::HexNumber) => Ok(Instruction::MovLitMem(lhs, rhs).into()),
        _ => return unexpected_token(source.as_ref(), &rhs_token),
    }
}

mod precedences {
    use miette::Result;

    use crate::parser::ast::Operator;

    pub const BASE: u8 = 0;
    pub const ADD: u8 = 1;
    pub const MUL: u8 = 2;
    pub const GROUP: u8 = 3;

    pub fn from_operator(operator: Operator) -> Result<u8> {
        match operator {
            Operator::Add => Ok(ADD),
            Operator::Sub => Ok(ADD),
            Operator::Mul => Ok(MUL),
        }
    }
}

pub fn parse_address_expr<S: AsRef<str>>(source: S, lexer: &mut Lexer, help: S, message: S) -> Result<Statement> {
    expect(Kind::Ampersand, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;
    expect(Kind::LBracket, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;

    println!("asda");

    let value = parse_expr(source.as_ref(), lexer, precedences::BASE)?;

    expect(Kind::RBracket, lexer, source.as_ref(), help.as_ref(), message.as_ref())?;
    Ok(Statement::Address(Box::new(value)))
}

pub fn parse_expr<S: AsRef<str>>(source: S, lexer: &mut Lexer, precedence: u8) -> Result<Statement> {
    let mut lhs = match peek(source.as_ref(), lexer)?.kind {
        Kind::LParen => {
            lexer.next().transpose()?;
            let value = parse_expr(source.as_ref(), lexer, precedences::GROUP)?;
            expect(Kind::RParen, lexer, source.as_ref(), "a", "b")?;
            value
        }
        Kind::HexNumber => Statement::HexLiteral(parse_hex_lit(source.as_ref(), lexer, "a", "b")?),
        _ => todo!(),
    };

    loop {
        match peek(source.as_ref(), lexer)?.kind {
            Kind::RParen => break,
            kind if !kind.is_operator() => todo!(),
            _ => {}
        }

        let operator = Operator::try_from(lexer.next().transpose()?.unwrap())?;
        let operator_precedence = precedences::from_operator(operator)?;

        if operator_precedence < precedence {
            break;
        }

        let rhs = parse_expr(source.as_ref(), lexer, operator_precedence)?;
        lhs = Statement::BinaryOp {
            lhs: Box::new(lhs),
            operator,
            rhs: Box::new(rhs),
        }
    }

    Ok(lhs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> Statement {
        let mut lexer = Lexer::new(input);
        parse_mov(input, &mut lexer).unwrap()
    }

    #[test]
    fn test_mov_lit_reg() {
        let input = "mov r1, $c0d3";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_mov_reg_reg() {
        let input = "mov r1, r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_mov_reg_mem() {
        let input = "mov &[$c0d3], r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_mov_mem_reg() {
        let input = "mov r4, &[$c0d3]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_mov_lit_mem() {
        let input = "mov &[$c0d3], $ffe3";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_mov_var_reg() {
        let input = "mov r3, !var";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_mov_mem_reg_expr() {
        let input = "mov r1, &[r1 + r2]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    #[should_panic]
    fn test_invalid_mem_syntax() {
        let input = "mov r3, &abcd";
        run_instruction(input);
    }
}
