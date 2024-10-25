use crate::lexer::{Kind, Lexer};
use crate::parser::ast::{Instruction, Statement};
use crate::parser::common::{parse_keyword, parse_register};
use crate::parser::Result;

pub fn parse_inc<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    parse_keyword(source.as_ref(), lexer, Kind::Inc)?;
    let value = Statement::Register(parse_register(source.as_ref(), lexer)?);
    Ok(Instruction::Inc(value).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> Statement {
        let mut lexer = Lexer::new(input);
        parse_inc(input, &mut lexer).unwrap()
    }

    #[test]
    fn test_inc_reg() {
        let input = "inc r1";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }
}
