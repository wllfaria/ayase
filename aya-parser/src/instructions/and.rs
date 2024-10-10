use nom::branch::alt;
use nom::IResult;

use crate::formats::{lit_reg, reg_reg};
use crate::types::Instruction;

pub fn parse_and(input: &str) -> IResult<&str, Instruction> {
    alt((
        |input| lit_reg(input, "and", Instruction::AndLitReg),
        |input| reg_reg(input, "and", Instruction::AndRegReg),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_and(input).unwrap()
    }

    #[test]
    fn test_and_lit_reg() {
        let input = "and r1, $c0d3";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_and_reg_reg() {
        let input = "and r1, r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
