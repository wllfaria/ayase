use crate::lexer::{Kind, Lexer, TransposeRef};
use crate::parser::ast::{Instruction, Statement};
use crate::parser::common::{expect, parse_hex_lit, parse_keyword, parse_register, parse_variable};
use crate::parser::error::{
    unexpected_eof, unexpected_token, ADDRESS_HELP, ADDRESS_MSG, COMMA_MSG, HEX_LIT_HELP, HEX_LIT_MSG,
};
use crate::parser::syntax::parse_simple_address;
use crate::parser::Result;

pub fn parse_mov<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    parse_keyword(source.as_ref(), lexer, Kind::Mov)?;

    let Ok(Some(token)) = lexer.peek().transpose() else {
        let Err(err) = lexer.next().transpose() else {
            return unexpected_eof(source.as_ref(), "unterminated import statement");
        };
        return Err(err);
    };
    let lhs_kind = token.kind;

    let lhs = match lhs_kind {
        Kind::Ident => Statement::Register(parse_register(source.as_ref(), lexer)?),
        Kind::Ampersand => parse_simple_address(source.as_ref(), lexer, ADDRESS_HELP, ADDRESS_MSG)?,
        _ => return unexpected_token(source.as_ref(), token),
    };

    expect(
        Kind::Comma,
        lexer,
        source.as_ref(),
        "missing a comma after left side of instruction",
        COMMA_MSG,
    )?;

    let Ok(Some(token)) = lexer.peek().transpose() else {
        let Err(err) = lexer.next().transpose() else {
            return unexpected_eof(source.as_ref(), "unterminated import statement");
        };
        return Err(err);
    };
    let token = token.clone();

    let rhs_kind = token.kind;
    let rhs = match rhs_kind {
        Kind::Ident => Statement::Register(parse_register(source.as_ref(), lexer)?),
        Kind::Bang => Statement::Var(parse_variable(source.as_ref(), lexer, "", "")?),
        Kind::HexNumber => Statement::HexLiteral(parse_hex_lit(source.as_ref(), lexer, HEX_LIT_HELP, HEX_LIT_MSG)?),
        Kind::Ampersand => parse_simple_address(source.as_ref(), lexer, ADDRESS_HELP, ADDRESS_MSG)?,
        _ => return unexpected_token(source.as_ref(), &token),
    };

    match (lhs_kind, rhs_kind) {
        (Kind::Ident, Kind::Ident) => Ok(Instruction::MovRegReg(lhs, rhs).into()),
        (Kind::Ident, Kind::Bang) => Ok(Instruction::MovLitReg(lhs, rhs).into()),
        (Kind::Ident, Kind::HexNumber) => Ok(Instruction::MovLitReg(lhs, rhs).into()),
        (Kind::Ident, Kind::Ampersand) => Ok(Instruction::MovMemReg(lhs, rhs).into()),
        (Kind::Ampersand, Kind::Ident) => Ok(Instruction::MovRegMem(lhs, rhs).into()),
        (Kind::Ampersand, Kind::Bang) => Ok(Instruction::MovLitMem(lhs, rhs).into()),
        (Kind::Ampersand, Kind::HexNumber) => Ok(Instruction::MovLitMem(lhs, rhs).into()),
        _ => return unexpected_token(source.as_ref(), &token),
    }
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
    #[should_panic]
    fn test_invalid_mem_syntax() {
        let input = "mov r3, &abcd";
        run_instruction(input);
    }
}
