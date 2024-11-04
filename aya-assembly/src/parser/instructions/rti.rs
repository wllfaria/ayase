use crate::lexer::{Kind, Lexer};
use crate::parser::ast::{Instruction, Statement};
use crate::parser::common::parse_keyword;
use crate::parser::Result;

pub fn parse_rti<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    let offset = parse_keyword(source.as_ref(), lexer, Kind::Rti)?;
    Ok(Instruction::Rti(offset).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> Statement {
        let mut lexer = Lexer::new(input);
        parse_rti(input, &mut lexer).unwrap()
    }

    #[test]
    fn test_hlt() {
        let input = "rti";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }
}
