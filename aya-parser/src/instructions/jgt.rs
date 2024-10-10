use nom::branch::alt;
use nom::IResult;

use crate::formats::{lit_mem, reg_mem};
use crate::types::Instruction;

pub fn parse_jgt(input: &str) -> IResult<&str, Instruction> {
    alt((
        |input| lit_mem(input, "jgt", Instruction::JgtLit),
        |input| reg_mem(input, "jgt", Instruction::JgtReg),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_jgt(input).unwrap()
    }

    #[test]
    fn test_jgt_reg() {
        let input = "jgt &c0d3, r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_jgt_lit() {
        let input = "jgt &c0d3, $0303";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
