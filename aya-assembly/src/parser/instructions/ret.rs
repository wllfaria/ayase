use crate::lexer::{Kind, Lexer};
use crate::parser::ast::{Instruction, Statement};
use crate::parser::common::parse_keyword;
use crate::parser::Result;

pub fn parse_ret<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    parse_keyword(source.as_ref(), lexer, Kind::Ret)?;
    Ok(Instruction::Ret.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> Statement {
        let mut lexer = Lexer::new(input);
        parse_ret(input, &mut lexer).unwrap()
    }

    #[test]
    fn test_ret() {
        let input = "ret";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }
}
