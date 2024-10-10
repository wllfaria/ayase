use nom::IResult;

use crate::formats::no_arg;
use crate::types::Instruction;

pub fn parse_ret(input: &str) -> IResult<&str, Instruction> {
    no_arg(input, "ret", || Instruction::Ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_ret(input).unwrap()
    }

    #[test]
    fn test_ret() {
        let input = "ret";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
