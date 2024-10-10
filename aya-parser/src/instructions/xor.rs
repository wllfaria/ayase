use nom::branch::alt;
use nom::IResult;

use crate::formats::{lit_reg, reg_reg};
use crate::types::Instruction;

pub fn parse_xor(input: &str) -> IResult<&str, Instruction> {
    alt((
        |input| lit_reg(input, "xor", Instruction::XorLitReg),
        |input| reg_reg(input, "xor", Instruction::XorRegReg),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_xor(input).unwrap()
    }

    #[test]
    fn test_xor_lit_reg() {
        let input = "xor r1, $c0d3";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_xor_reg_reg() {
        let input = "xor r1, r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
