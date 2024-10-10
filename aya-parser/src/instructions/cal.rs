use nom::branch::alt;
use nom::IResult;

use crate::formats::{single_lit, single_reg};
use crate::types::Instruction;

pub fn parse_cal(input: &str) -> IResult<&str, Instruction> {
    alt((
        |input| single_lit(input, "cal", Instruction::CalLit),
        |input| single_reg(input, "cal", Instruction::CalReg),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_cal(input).unwrap()
    }

    #[test]
    fn test_cal_reg() {
        let input = "cal r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_cal_lit() {
        let input = "cal $0303";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
