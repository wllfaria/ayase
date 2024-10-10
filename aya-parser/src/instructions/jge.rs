use nom::branch::alt;
use nom::IResult;

use crate::formats::{lit_mem, reg_mem};
use crate::types::Instruction;

pub fn parse_jge(input: &str) -> IResult<&str, Instruction> {
    alt((
        |input| lit_mem(input, "jge", Instruction::JgeLit),
        |input| reg_mem(input, "jge", Instruction::JgeReg),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_jge(input).unwrap()
    }

    #[test]
    fn test_jge_reg() {
        let input = "jge &c0d3, r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_jge_lit() {
        let input = "jge &c0d3, $0303";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
