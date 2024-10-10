use nom::IResult;

use crate::formats::single_reg;
use crate::types::Instruction;

pub fn parse_dec(input: &str) -> IResult<&str, Instruction> {
    single_reg(input, "dec", Instruction::Dec)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_dec(input).unwrap()
    }

    #[test]
    fn test_dec_reg() {
        let input = "dec r1";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
