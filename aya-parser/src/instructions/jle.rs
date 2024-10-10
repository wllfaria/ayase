use nom::branch::alt;
use nom::IResult;

use crate::formats::{lit_mem, reg_mem};
use crate::types::Instruction;

pub fn parse_jle(input: &str) -> IResult<&str, Instruction> {
    alt((
        |input| lit_mem(input, "jle", Instruction::JleLit),
        |input| reg_mem(input, "jle", Instruction::JleReg),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_jle(input).unwrap()
    }

    #[test]
    fn test_jle_reg() {
        let input = "jle &c0d3, r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_jle_lit() {
        let input = "jle &c0d3, $0303";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
