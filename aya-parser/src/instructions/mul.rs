use nom::branch::alt;
use nom::IResult;

use crate::formats::{lit_reg, reg_reg};
use crate::types::Instruction;

pub fn parse_mul(input: &str) -> IResult<&str, Instruction> {
    alt((
        |input| lit_reg(input, "mul", Instruction::MulLitReg),
        |input| reg_reg(input, "mul", Instruction::MulRegReg),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_mul(input).unwrap()
    }

    #[test]
    fn test_mul_lit_reg() {
        let input = "mul r1, $c0d3";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_mul_reg_reg() {
        let input = "mul r1, r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
