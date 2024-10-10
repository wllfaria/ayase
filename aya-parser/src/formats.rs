use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{char, space0, space1};
use nom::sequence::preceded;
use nom::IResult;

use crate::common::{address, hex_literal, register};
use crate::expressions::square_bracketed_expr;
use crate::types::{Atom, Instruction};

pub fn lit_reg<'parser, M: Fn(Atom<'parser>, Atom<'parser>) -> Instruction<'parser>>(
    input: &'parser str,
    prefix: &'parser str,
    mapper: M,
) -> IResult<&'parser str, Instruction<'parser>> {
    let (input, _) = tag_no_case(prefix)(input)?;
    let (input, _) = space1(input)?;

    let (input, lhs) = register(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = space0(input)?;

    let (input, rhs) = alt((hex_literal, square_bracketed_expr))(input)?;
    let (input, _) = space0(input)?;

    Ok((input, mapper(lhs, rhs)))
}

pub fn reg_reg<'parser, M: Fn(Atom<'parser>, Atom<'parser>) -> Instruction<'parser>>(
    input: &'parser str,
    prefix: &'parser str,
    mapper: M,
) -> IResult<&'parser str, Instruction<'parser>> {
    let (input, _) = tag_no_case(prefix)(input)?;
    let (input, _) = space1(input)?;

    let (input, lhs) = register(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = space0(input)?;

    let (input, rhs) = register(input)?;
    let (input, _) = space0(input)?;

    Ok((input, mapper(lhs, rhs)))
}

pub fn reg_mem<'parser, M: Fn(Atom<'parser>, Atom<'parser>) -> Instruction<'parser>>(
    input: &'parser str,
    prefix: &'parser str,
    mapper: M,
) -> IResult<&'parser str, Instruction<'parser>> {
    let (input, _) = tag_no_case(prefix)(input)?;
    let (input, _) = space1(input)?;

    let (input, lhs) = alt((address, preceded(char('&'), square_bracketed_expr)))(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = space0(input)?;

    let (input, rhs) = register(input)?;
    let (input, _) = space0(input)?;

    Ok((input, mapper(lhs, rhs)))
}

pub fn mem_reg<'parser, M: Fn(Atom<'parser>, Atom<'parser>) -> Instruction<'parser>>(
    input: &'parser str,
    prefix: &'parser str,
    mapper: M,
) -> IResult<&'parser str, Instruction<'parser>> {
    let (input, _) = tag_no_case(prefix)(input)?;
    let (input, _) = space1(input)?;

    let (input, lhs) = register(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = space0(input)?;

    let (input, rhs) = alt((address, preceded(char('&'), square_bracketed_expr)))(input)?;
    let (input, _) = space0(input)?;

    Ok((input, mapper(lhs, rhs)))
}

pub fn lit_mem<'parser, M: Fn(Atom<'parser>, Atom<'parser>) -> Instruction<'parser>>(
    input: &'parser str,
    prefix: &'parser str,
    mapper: M,
) -> IResult<&'parser str, Instruction<'parser>> {
    let (input, _) = tag_no_case(prefix)(input)?;
    let (input, _) = space1(input)?;

    let (input, lhs) = alt((address, preceded(char('&'), square_bracketed_expr)))(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = space0(input)?;

    let (input, rhs) = alt((hex_literal, square_bracketed_expr))(input)?;
    let (input, _) = space0(input)?;

    Ok((input, mapper(lhs, rhs)))
}

pub fn reg_ptr_reg<'parser, M: Fn(Atom<'parser>, Atom<'parser>) -> Instruction<'parser>>(
    input: &'parser str,
    prefix: &'parser str,
    mapper: M,
) -> IResult<&'parser str, Instruction<'parser>> {
    let (input, _) = tag_no_case(prefix)(input)?;
    let (input, _) = space1(input)?;

    let (input, lhs) = register(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = space0(input)?;

    let (input, rhs) = preceded(char('&'), register)(input)?;
    let (input, _) = space0(input)?;

    Ok((input, mapper(lhs, rhs)))
}

pub fn single_reg<'parser, M: Fn(Atom<'parser>) -> Instruction<'parser>>(
    input: &'parser str,
    prefix: &'parser str,
    mapper: M,
) -> IResult<&'parser str, Instruction<'parser>> {
    let (input, _) = tag_no_case(prefix)(input)?;
    let (input, _) = space1(input)?;

    let (input, reg) = register(input)?;
    let (input, _) = space0(input)?;

    Ok((input, mapper(reg)))
}

pub fn single_lit<'parser, M: Fn(Atom<'parser>) -> Instruction<'parser>>(
    input: &'parser str,
    prefix: &'parser str,
    mapper: M,
) -> IResult<&'parser str, Instruction<'parser>> {
    let (input, _) = tag_no_case(prefix)(input)?;
    let (input, _) = space1(input)?;

    let (input, lit) = alt((hex_literal, square_bracketed_expr))(input)?;
    let (input, _) = space0(input)?;

    Ok((input, mapper(lit)))
}

pub fn no_arg<'parser, M: Fn() -> Instruction<'parser>>(
    input: &'parser str,
    prefix: &'parser str,
    mapper: M,
) -> IResult<&'parser str, Instruction<'parser>> {
    let (input, _) = tag_no_case(prefix)(input)?;
    let (input, _) = space0(input)?;

    Ok((input, mapper()))
}
