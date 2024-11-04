use crate::lexer::{Kind, Lexer};
use crate::parser::ast::{Instruction, Statement};
use crate::parser::common::parse_keyword;
use crate::parser::error::{ADDRESS_HELP, ADDRESS_MSG};
use crate::parser::expressions::parse_address_expr;
use crate::parser::Result;

pub fn parse_jmp<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    parse_keyword(source.as_ref(), lexer, Kind::Jmp)?;

    let lhs = parse_address_expr(source.as_ref(), lexer, ADDRESS_HELP, ADDRESS_MSG)?;

    Ok(Instruction::Jmp(lhs).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> Statement {
        let mut lexer = Lexer::new(input);
        parse_jmp(input, &mut lexer).unwrap()
    }

    #[test]
    fn test_jmp_simple() {
        let input = "jmp &[$c0d3]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_jmp_expr() {
        let input = "jmp &[$c0d3 + r2]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }
}
