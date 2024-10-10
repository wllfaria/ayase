use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::space0;
use nom::IResult;

use crate::common::{get_precedence, hex_literal, is_terminator, operator, peek, precedences, register, variable};
use crate::types::Atom;

fn expr_group(input: &str) -> IResult<&str, Atom> {
    let (input, _) = tag("(")(input)?;
    let (input, _) = space0(input)?;

    let (input, expr) = parse_expr(input, precedences::BASE)?;
    let (input, _) = space0(input)?;

    let (input, _) = tag(")")(input)?;
    let (input, _) = space0(input)?;

    Ok((input, expr))
}

fn expr_item(input: &str) -> IResult<&str, Atom> {
    alt((hex_literal, register, variable, expr_group))(input)
}

fn parse_expr(input: &str, min_precedence: u8) -> IResult<&str, Atom> {
    let (input, mut lhs) = expr_item(input)?;
    let (input, _) = space0(input)?;

    let mut full_input = input;

    loop {
        let (_, next) = peek(full_input)?;
        if is_terminator(next) {
            break;
        }

        let (input, operator) = operator(full_input)?;
        let (input, _) = space0(input)?;

        let Atom::Operator(operator) = operator else {
            unreachable!();
        };
        let precedence = get_precedence(operator);

        if precedence <= min_precedence {
            break;
        }

        let (input, rhs) = parse_expr(input, precedence)?;
        let (input, _) = space0(input)?;

        full_input = input;

        lhs = Atom::BinaryOp {
            lhs: Box::new(lhs),
            operator,
            rhs: Box::new(rhs),
        }
    }

    Ok((full_input, lhs))
}

pub fn square_bracketed_expr(input: &str) -> IResult<&str, Atom> {
    let (input, _) = tag("[")(input)?;
    let (input, _) = space0(input)?;

    let (input, expr) = parse_expr(input, precedences::BASE)?;
    let (input, _) = space0(input)?;

    let (input, _) = tag("]")(input)?;
    let (input, _) = space0(input)?;

    Ok((input, expr))
}
