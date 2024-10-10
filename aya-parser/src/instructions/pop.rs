use nom::IResult;

use crate::formats::single_reg;
use crate::types::Instruction;

pub fn parse_pop(input: &str) -> IResult<&str, Instruction> {
    single_reg(input, "pop", Instruction::Pop)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_pop(input).unwrap()
    }

    #[test]
    fn test_pop_reg() {
        let input = "pop r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
