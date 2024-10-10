use nom::IResult;

use crate::formats::single_reg;
use crate::types::Instruction;

pub fn parse_inc(input: &str) -> IResult<&str, Instruction> {
    single_reg(input, "inc", Instruction::Inc)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_inc(input).unwrap()
    }

    #[test]
    fn test_inc_reg() {
        let input = "inc r1";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
