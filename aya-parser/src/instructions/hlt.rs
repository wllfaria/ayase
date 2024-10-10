use nom::IResult;

use crate::formats::no_arg;
use crate::types::Instruction;

pub fn parse_hlt(input: &str) -> IResult<&str, Instruction> {
    no_arg(input, "hlt", || Instruction::Hlt)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_hlt(input).unwrap()
    }

    #[test]
    fn test_hlt() {
        let input = "hlt";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
