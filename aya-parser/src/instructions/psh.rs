use nom::branch::alt;
use nom::IResult;

use crate::formats::{single_lit, single_reg};
use crate::types::Instruction;

pub fn parse_psh(input: &str) -> IResult<&str, Instruction> {
    alt((
        |input| single_lit(input, "psh", Instruction::PshLit),
        |input| single_reg(input, "psh", Instruction::PshReg),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_psh(input).unwrap()
    }

    #[test]
    fn test_psh_reg() {
        let input = "psh r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_psh_lit() {
        let input = "psh $0303";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
