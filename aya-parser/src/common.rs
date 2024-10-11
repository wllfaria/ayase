use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{alpha1, alphanumeric1, anychar, char, hex_digit1, multispace0, one_of};
use nom::combinator::{map, peek as nom_peek, recognize};
use nom::multi::many0_count;
use nom::sequence::{pair, terminated};
use nom::IResult;

use crate::types::{Atom, Operator};

pub mod precedences {
    pub const BASE: u8 = 0;
    pub const SUM: u8 = 1;
    pub const MUL: u8 = 2;
}

pub fn get_precedence(operator: Operator) -> u8 {
    match operator {
        Operator::Add | Operator::Sub => precedences::SUM,
        Operator::Mul => precedences::MUL,
    }
}

pub fn is_terminator(ch: char) -> bool {
    matches!(ch, ']' | ')')
}

pub fn peek(input: &str) -> IResult<&str, char> {
    nom_peek(anychar)(input)
}

pub fn general_register(input: &str) -> IResult<&str, &str> {
    recognize(pair(tag_no_case("r"), one_of("12345678")))(input)
}

pub fn register(input: &str) -> IResult<&str, Atom> {
    map(
        alt((
            general_register,
            tag_no_case("ip"),
            tag_no_case("sp"),
            tag_no_case("fp"),
            tag_no_case("ret"),
        )),
        Atom::Register,
    )(input)
}

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

pub fn variable(input: &str) -> IResult<&str, Atom> {
    let (input, _) = tag("!")(input)?;
    map(identifier, Atom::Var)(input)
}

pub fn label(input: &str) -> IResult<&str, Atom> {
    let (input, _) = multispace0(input)?;
    let (input, label) = map(terminated(identifier, char(':')), Atom::Label)(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, label))
}

pub fn hex_literal(input: &str) -> IResult<&str, Atom> {
    let (input, _) = tag("$")(input)?;
    map(hex_digit1, Atom::HexLiteral)(input)
}

pub fn address(input: &str) -> IResult<&str, Atom> {
    let (input, _) = tag("&")(input)?;
    map(hex_digit1, Atom::Address)(input)
}

pub fn operator(input: &str) -> IResult<&str, Atom> {
    map(
        alt((
            map(char('+'), |_| Operator::Add),
            map(char('-'), |_| Operator::Sub),
            map(char('*'), |_| Operator::Mul),
        )),
        Atom::Operator,
    )(input)
}
