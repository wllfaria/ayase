use nom::branch::alt;
use nom::IResult;

use crate::formats::{lit_mem, reg_mem};
use crate::types::Instruction;

pub fn parse_jne(input: &str) -> IResult<&str, Instruction> {
    alt((
        |input| lit_mem(input, "jne", Instruction::JneLit),
        |input| reg_mem(input, "jne", Instruction::JneReg),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_jne(input).unwrap()
    }

    #[test]
    fn test_jne_reg() {
        let input = "jne &c0d3, r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_jne_lit() {
        let input = "jne &c0d3, $0303";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
