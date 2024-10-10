use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1, newline, one_of, space0, space1};
use nom::combinator::{eof, recognize, value};
use nom::sequence::{pair, terminated, tuple};
use nom::IResult;

//fn register_name(input: &str) -> IResult<&str, &str> {
//    recognize(pair(tag("r"), one_of("12345678")))(input)
//}
//
//fn address(input: &str) -> IResult<&str, &str> {
//    todo!();
//}
//
//fn newline_or_eof(input: &str) -> IResult<&str, &str> {
//    alt((value("", newline), eof))(input)
//}
//
//fn mov_lhs(input: &str) -> IResult<&str, &str> {
//    terminated(alt((register_name, register_name)), multispace1)(input)
//}
//
//fn mov_rhs(input: &str) -> IResult<&str, &str> {
//    terminated(
//        alt((register_name, register_name, register_name)),
//        tuple((space0, newline_or_eof, multispace0)),
//    )(input)
//}
//
//fn parse_mov(input: &str) -> IResult<&str, (&str, &str)> {
//    let (input, _) = terminated(tag("mov"), space1)(input)?;
//    let (input, result) = tuple((mov_lhs, mov_rhs))(input)?;
//    Ok((input, result))
//}

fn upper_or_lower_str(str: &str) -> IResult<&str, &str> {
    tag_no_case(str)(str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_mov() {
        let input = "mov r1 r3";
        let (_, (r1, r2)) = parse_mov(input).unwrap();
        assert!(r1 == "r1");
        assert!(r2 == "r3");
    }
}
