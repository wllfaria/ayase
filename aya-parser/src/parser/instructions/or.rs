use crate::ast::{Instruction, Statement};
use crate::lexer::{Kind, Lexer, TransposeRef};
use crate::parser::common::{expect, parse_hex_lit, parse_keyword, parse_register};
use crate::parser::error::{unexpected_eof, unexpected_token, COMMA_MSG, HEX_LIT_HELP, HEX_LIT_MSG};
use crate::parser::Result;

pub fn parse_or<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    parse_keyword(source.as_ref(), lexer, Kind::Or)?;

    let lhs = Statement::Register(parse_register(source.as_ref(), lexer)?);

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
        _ => return unexpected_token(source.as_ref(), token),
    };

    match kind {
        Kind::Ident => Ok(Instruction::OrRegReg(lhs, rhs).into()),
        Kind::HexNumber => Ok(Instruction::OrLitReg(lhs, rhs).into()),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> Statement {
        let mut lexer = Lexer::new(input);
        parse_or(input, &mut lexer).unwrap()
    }

    #[test]
    fn test_or_lit_reg() {
        let input = "or r1, $c0d3";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_or_reg_reg() {
        let input = "or r1, r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }
}
