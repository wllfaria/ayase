use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{char, space0, space1};
use nom::sequence::preceded;
use nom::IResult;

use crate::common::{address, hex_literal, register};
use crate::expressions::square_bracketed_expr;
use crate::types::Instruction;

fn mov_lit_reg(input: &str) -> IResult<&str, Instruction> {
    let (input, _) = tag_no_case("mov")(input)?;
    let (input, _) = space1(input)?;

    let (input, lhs) = register(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = space0(input)?;

    let (input, rhs) = alt((hex_literal, square_bracketed_expr))(input)?;
    let (input, _) = space0(input)?;

    Ok((input, Instruction::MovLitReg(lhs, rhs)))
}

fn mov_reg_reg(input: &str) -> IResult<&str, Instruction> {
    let (input, _) = tag_no_case("mov")(input)?;
    let (input, _) = space1(input)?;

    let (input, lhs) = register(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = space0(input)?;

    let (input, rhs) = register(input)?;
    let (input, _) = space0(input)?;

    Ok((input, Instruction::MovRegReg(lhs, rhs)))
}

fn mov_reg_mem(input: &str) -> IResult<&str, Instruction> {
    let (input, _) = tag_no_case("mov")(input)?;
    let (input, _) = space1(input)?;

    let (input, lhs) = alt((address, preceded(char('&'), square_bracketed_expr)))(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = space0(input)?;

    let (input, rhs) = register(input)?;
    let (input, _) = space0(input)?;

    Ok((input, Instruction::MovRegMem(lhs, rhs)))
}

fn mov_mem_reg(input: &str) -> IResult<&str, Instruction> {
    let (input, _) = tag_no_case("mov")(input)?;
    let (input, _) = space1(input)?;

    let (input, lhs) = register(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = space0(input)?;

    let (input, rhs) = alt((address, preceded(char('&'), square_bracketed_expr)))(input)?;
    let (input, _) = space0(input)?;

    Ok((input, Instruction::MovMemReg(lhs, rhs)))
}

fn mov_lit_mem(input: &str) -> IResult<&str, Instruction> {
    let (input, _) = tag_no_case("mov")(input)?;
    let (input, _) = space1(input)?;

    let (input, lhs) = alt((address, preceded(char('&'), square_bracketed_expr)))(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = space0(input)?;

    let (input, rhs) = alt((hex_literal, square_bracketed_expr))(input)?;
    let (input, _) = space0(input)?;

    Ok((input, Instruction::MovLitMem(lhs, rhs)))
}

fn mov_reg_ptr_reg(input: &str) -> IResult<&str, Instruction> {
    let (input, _) = tag_no_case("mov")(input)?;
    let (input, _) = space1(input)?;

    let (input, lhs) = register(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = space0(input)?;

    let (input, rhs) = preceded(char('&'), register)(input)?;
    let (input, _) = space0(input)?;

    Ok((input, Instruction::MovRegPtrReg(lhs, rhs)))
}

pub fn parse_mov(input: &str) -> IResult<&str, Instruction> {
    alt((
        mov_lit_reg,
        mov_reg_reg,
        mov_reg_mem,
        mov_mem_reg,
        mov_lit_mem,
        mov_reg_ptr_reg,
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Atom, Operator};

    fn run_instruction(input: &str) {
        let (input, instruction) = parse_mov(input).unwrap();
        insta::assert_debug_snapshot!(instruction);
        assert!(input.is_empty());
    }

    #[test]
    fn test_mov_lit_reg() {
        let input = "mov r1, $c0d3";
        run_instruction(input);
    }

    #[test]
    fn test_mov_reg_reg() {
        let input = "mov r1, r2";
        run_instruction(input);
    }

    #[test]
    fn test_mov_reg_mem() {
        let input = "mov &c0d3, r2";
        run_instruction(input);
    }

    #[test]
    fn test_mov_mem_reg() {
        let input = "mov r4, &c0d3";
        run_instruction(input);
    }

    #[test]
    fn test_mov_lit_mem() {
        let input = "mov &c0d3, $ffe3";
        run_instruction(input);
    }

    #[test]
    fn test_mov_reg_ptr_reg() {
        let input = "mov r3, &r4";
        run_instruction(input);
    }

    #[test]
    fn parse_complex_mov_reg_lit() {
        let input = "mov r1, [$42 + !loc - ($05 * ($31 + !var) - $07)]";
        run_instruction(input);
    }
}
