use nom::branch::alt;
use nom::IResult;

use crate::formats::{lit_mem, reg_mem};
use crate::types::Instruction;

pub fn parse_jlt(input: &str) -> IResult<&str, Instruction> {
    alt((
        |input| lit_mem(input, "jlt", Instruction::JltLit),
        |input| reg_mem(input, "jlt", Instruction::JltReg),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_jlt(input).unwrap()
    }

    #[test]
    fn test_jlt_reg() {
        let input = "jlt &c0d3, r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_jlt_lit() {
        let input = "jlt &c0d3, $0303";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
