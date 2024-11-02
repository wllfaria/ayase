use super::mov::peek;
use crate::lexer::{Kind, Lexer, TransposeRef};
use crate::parser::ast::{Instruction, Statement};
use crate::parser::common::{expect, parse_hex_lit, parse_keyword, parse_register, parse_variable};
use crate::parser::error::{unexpected_eof, unexpected_token, COMMA_MSG, HEX_LIT_HELP, HEX_LIT_MSG};
use crate::parser::Result;

pub fn parse_add<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    parse_keyword(source.as_ref(), lexer, Kind::Add)?;

    let lhs = Statement::Register(parse_register(source.as_ref(), lexer)?);

    expect(
        Kind::Comma,
        lexer,
        source.as_ref(),
        "missing a comma after left side of instruction",
        COMMA_MSG,
    )?;

    let token = peek(source.as_ref(), lexer)?;
    let rhs = match token.kind {
        Kind::Ident => Statement::Register(parse_register(source.as_ref(), lexer)?),
        Kind::HexNumber => Statement::HexLiteral(parse_hex_lit(source.as_ref(), lexer, HEX_LIT_HELP, HEX_LIT_MSG)?),
        Kind::Bang => Statement::Var(parse_variable(source.as_ref(), lexer, "", "")?),
        _ => return unexpected_token(source.as_ref(), &token),
    };

    match token.kind {
        Kind::Ident => Ok(Instruction::AddRegReg(lhs, rhs).into()),
        Kind::HexNumber => Ok(Instruction::AddLitReg(lhs, rhs).into()),
        Kind::Bang => Ok(Instruction::AddLitReg(lhs, rhs).into()),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> Statement {
        let mut lexer = Lexer::new(input);
        parse_add(input, &mut lexer).unwrap()
    }

    #[test]
    fn test_add_lit_reg() {
        let input = "add r1, $c0d3";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_add_reg_reg() {
        let input = "add r1, r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }
}
