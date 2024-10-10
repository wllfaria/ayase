use nom::branch::alt;
use nom::IResult;

use crate::formats::{lit_mem, lit_reg, mem_reg, reg_mem, reg_ptr_reg, reg_reg};
use crate::types::Instruction;

pub fn parse_mov(input: &str) -> IResult<&str, Instruction> {
    alt((
        |input| lit_reg(input, "mov", Instruction::MovLitReg),
        |input| reg_reg(input, "mov", Instruction::MovRegReg),
        |input| reg_mem(input, "mov", Instruction::MovRegMem),
        |input| mem_reg(input, "mov", Instruction::MovMemReg),
        |input| lit_mem(input, "mov", Instruction::MovLitMem),
        |input| reg_ptr_reg(input, "mov", Instruction::MovRegPtrReg),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> (&str, Instruction) {
        parse_mov(input).unwrap()
    }

    #[test]
    fn test_mov_lit_reg() {
        let input = "mov r1, $c0d3";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_mov_reg_reg() {
        let input = "mov r1, r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_mov_reg_mem() {
        let input = "mov &c0d3, r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_mov_mem_reg() {
        let input = "mov r4, &c0d3";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_mov_lit_mem() {
        let input = "mov &c0d3, $ffe3";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_mov_reg_ptr_reg() {
        let input = "mov r3, &r4";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_complex_mov_reg_lit() {
        let input = "mov r1, [$42 + !loc - ($05 * ($31 + !var) - $07)]";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_complex_mov_reg_mem() {
        let input = "mov &[$42 + !loc - ($05 * ($31 + !var) - $07)], r2";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_complex_mov_mem_reg() {
        let input = "mov r1, &[$42 + !loc - ($05 * ($31 + !var) - $07)]";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_complex_mov_lit_mem() {
        let input = "mov &[$42 + !loc - ($05 * ($31 + !var) - $07)], $ffee";
        let (input, instruction) = run_instruction(input);
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }
}
