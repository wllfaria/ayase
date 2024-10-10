use nom::IResult;

use crate::formats::single_reg;
use crate::types::Instruction;

pub fn parse_not(input: &str) -> IResult<&str, Instruction> {
    single_reg(input, "not", Instruction::Not)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_not(input).unwrap()
    }

    #[test]
    fn test_not_reg() {
        let input = "not r1";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
