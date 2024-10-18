use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{alpha1, alphanumeric1, anychar, char, hex_digit1, multispace0, one_of, space0};
use nom::combinator::{map, opt, peek as nom_peek, recognize};
use nom::multi::{many0_count, separated_list1};
use nom::sequence::{delimited, pair, terminated};
use nom::IResult;

use crate::types::{Ast, Operator};

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

pub fn ws<'parser, O, F: Fn(&'parser str) -> IResult<&'parser str, O>>(
    inner: F,
) -> impl FnMut(&'parser str) -> IResult<&'parser str, O> {
    delimited(space0, inner, space0)
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

pub fn register(input: &str) -> IResult<&str, Ast> {
    map(
        alt((
            general_register,
            tag_no_case("ip"),
            tag_no_case("sp"),
            tag_no_case("fp"),
            tag_no_case("ret"),
        )),
        Ast::Register,
    )(input)
}

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

pub fn variable(input: &str) -> IResult<&str, Ast> {
    let (input, _) = tag("!")(input)?;
    map(identifier, Ast::Var)(input)
}

pub fn label(input: &str) -> IResult<&str, Ast> {
    let (input, _) = multispace0(input)?;
    let (input, label) = map(terminated(identifier, char(':')), Ast::Label)(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, label))
}

pub fn constant(input: &str) -> IResult<&str, Ast> {
    let (input, _) = multispace0(input)?;
    let (input, exported) = opt(char('+'))(input)?;
    let exported = exported.is_some();
    let (input, _) = tag("const")(input)?;
    let (input, _) = space0(input)?;

    let (input, name) = identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;
    let (input, val) = hex_literal(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        Ast::Const {
            name,
            exported,
            value: Box::new(val),
        },
    ))
}

pub fn data(input: &str) -> IResult<&str, Ast> {
    let (input, _) = multispace0(input)?;
    let (input, exported) = opt(char('+'))(input)?;
    let exported = exported.is_some();

    let (input, _) = tag("data")(input)?;
    let (input, size) = map(alt((tag("8"), tag("16"))), |s: &str| s.parse::<u8>().unwrap())(input)?;

    let (input, _) = space0(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('{')(input)?;
    let (input, _) = space0(input)?;

    let (input, values) = terminated(
        separated_list1(delimited(space0, tag(","), space0), hex_literal),
        opt(tag(",")),
    )(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char('}')(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        Ast::Data {
            name,
            size,
            exported,
            values,
        },
    ))
}

pub fn hex_literal(input: &str) -> IResult<&str, Ast> {
    let (input, _) = tag("$")(input)?;
    map(hex_digit1, Ast::HexLiteral)(input)
}

pub fn address(input: &str) -> IResult<&str, Ast> {
    let (input, _) = tag("&")(input)?;
    map(hex_digit1, Ast::Address)(input)
}

pub fn operator(input: &str) -> IResult<&str, Ast> {
    map(
        alt((
            map(char('+'), |_| Operator::Add),
            map(char('-'), |_| Operator::Sub),
            map(char('*'), |_| Operator::Mul),
        )),
        Ast::Operator,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exported_data_8() {
        let input = "+data8 some_name = { $01,$02  , $03   , $04 }";
        let (_, result) = data(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_private_data_8() {
        let input = "data8 some_name = { $01,$02  , $03   , $04 }";
        let (_, result) = data(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_exported_data_16() {
        let input = "+data16 some_name = { $0102  , $0304 }";
        let (_, result) = data(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_private_data_16() {
        let input = "data16 some_name = { $0102  , $0304 }";
        let (_, result) = data(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_exported_const() {
        let input = "+const some_name = $0102";
        let (_, result) = constant(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_private_const() {
        let input = "const some_name = $0102";
        let (_, result) = constant(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }
}
