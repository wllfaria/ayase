use crate::lexer::{Kind, Lexer, TransposeRef};
use crate::parser::ast::{Instruction, Statement};
use crate::parser::common::{expect, parse_hex_lit, parse_keyword, parse_register, parse_variable};
use crate::parser::error::{
    ADDRESS_HELP, ADDRESS_MSG, BRACKETED_EXPR_HELP, BRACKETED_EXPR_MSG, COMMA_MSG, HEX_LIT_HELP, HEX_LIT_MSG, VAR_HELP,
    VAR_MSG,
};
use crate::parser::expressions::{parse_address_expr, parse_literal_expr};
use crate::parser::Result;
use crate::utils::{unexpected_eof, unexpected_token};

pub fn parse_jle<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    parse_keyword(source.as_ref(), lexer, Kind::Jle)?;

    let lhs = parse_address_expr(source.as_ref(), lexer, ADDRESS_HELP, ADDRESS_MSG)?;

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

    let kind = token.kind;
    let rhs = match kind {
        Kind::Ident => Statement::Register(parse_register(source.as_ref(), lexer)?),
        Kind::HexNumber => Statement::HexLiteral(parse_hex_lit(source.as_ref(), lexer, HEX_LIT_HELP, HEX_LIT_MSG)?),
        Kind::Bang => Statement::Var(parse_variable(source.as_ref(), lexer, VAR_HELP, VAR_MSG)?),
        Kind::LBracket => parse_literal_expr(source.as_ref(), lexer, BRACKETED_EXPR_HELP, BRACKETED_EXPR_MSG)?,
        _ => return unexpected_token(source.as_ref(), token),
    };

    match kind {
        Kind::Ident => Ok(Instruction::JleReg(lhs, rhs).into()),
        Kind::HexNumber => Ok(Instruction::JleLit(lhs, rhs).into()),
        Kind::Bang => Ok(Instruction::JleLit(lhs, rhs).into()),
        Kind::LBracket => Ok(Instruction::JleLit(lhs, rhs).into()),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> Statement {
        let mut lexer = Lexer::new(input);
        parse_jle(input, &mut lexer).unwrap()
    }

    #[test]
    fn test_jle_reg() {
        let input = "jle &[$c0d3], r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_jle_reg_expr() {
        let input = "jle &[$c0d3 + r2], r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_jle_lit() {
        let input = "jle &[$c0d3], $0303";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_jle_lit_var() {
        let input = "jle &[$c0d3], !var";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_jle_lit_expr() {
        let input = "jle &[$c0d3], [$0303 + r2]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_jle_lit_expr_both() {
        let input = "jle &[$c0d3 + r2], [$0303 + r2]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }
}
