use crate::lexer::{Kind, Lexer};
use crate::parser::ast::{Instruction, Statement};
use crate::parser::common::{parse_hex_lit, parse_keyword};
use crate::parser::error::{HEX_LIT_HELP, HEX_LIT_MSG};
use crate::parser::Result;

pub fn parse_int<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    parse_keyword(source.as_ref(), lexer, Kind::Int)?;
    let value = Statement::HexLiteral(parse_hex_lit(source.as_ref(), lexer, HEX_LIT_HELP, HEX_LIT_MSG)?);
    Ok(Instruction::Int(value).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> Statement {
        let mut lexer = Lexer::new(input);
        parse_int(input, &mut lexer).unwrap()
    }

    #[test]
    fn test_int() {
        let input = "int $03";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    #[should_panic]
    fn test_int_expr() {
        let input = "int [$03 + $1]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }
}
