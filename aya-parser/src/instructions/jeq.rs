use nom::branch::alt;
use nom::IResult;

use crate::formats::{lit_mem, reg_mem};
use crate::types::Instruction;

pub fn parse_jeq(input: &str) -> IResult<&str, Instruction> {
    alt((
        |input| lit_mem(input, "jeq", Instruction::JeqLit),
        |input| reg_mem(input, "jeq", Instruction::JeqReg),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_jeq(input).unwrap()
    }

    #[test]
    fn test_jeq_reg() {
        let input = "jeq &c0d3, r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_jeq_lit() {
        let input = "jeq &c0d3, $0303";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
