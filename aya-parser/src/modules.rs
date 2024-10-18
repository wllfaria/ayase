use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, multispace0, space0};
use nom::combinator::{map, recognize};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, separated_pair, terminated, tuple};
use nom::IResult;

use crate::common::{address, hex_literal, identifier, variable, ws};
use crate::Ast;

fn parse_path(input: &str) -> IResult<&str, &str> {
    delimited(
        tag("\""),
        recognize(many1(alt((alphanumeric1, tag("/"), tag("."), tag("_"))))),
        tag("\""),
    )(input)
}

fn field_accessor(input: &str) -> IResult<&str, Ast> {
    let (input, module) = identifier(input)?;
    let (input, _) = tag(".")(input)?;
    let (input, field) = identifier(input)?;

    Ok((input, Ast::FieldAccessor { module, field }))
}

fn bracketed_value(input: &str) -> IResult<&str, Ast> {
    let (input, _) = tag("[")(input)?;
    let (input, _) = space0(input)?;

    let (input, value) = alt((field_accessor, variable))(input)?;

    let (input, _) = tag("]")(input)?;
    let (input, _) = space0(input)?;

    Ok((input, value))
}

fn parse_import_variables(input: &str) -> IResult<&str, Vec<Ast>> {
    let (input, values) = many0(terminated(
        map(
            separated_pair(
                ws(identifier),
                ws(tag(":")),
                alt((ws(address), ws(hex_literal), ws(bracketed_value))),
            ),
            |(name, value)| Ast::ImportVar {
                name,
                value: Box::new(value),
            },
        ),
        tuple((tag(","), multispace0)),
    ))(input)?;

    Ok((input, values))
}

// TODO: this requires a trailing comma, it shouldn't
pub fn parse_import(input: &str) -> IResult<&str, Ast> {
    let (input, _) = multispace0(input)?;

    let (input, _) = tag("import")(input)?;
    let (input, _) = space0(input)?;

    let (input, path) = parse_path(input)?;
    let (input, _) = space0(input)?;

    let (input, name) = identifier(input)?;
    let (input, _) = space0(input)?;

    let (input, address) = address(input)?;
    let (input, _) = space0(input)?;

    let (input, _) = tag("{")(input)?;
    let (input, _) = multispace0(input)?;

    let (input, variables) = parse_import_variables(input)?;

    let (input, _) = tag("}")(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        Ast::Import {
            name,
            path,
            variables,
            address: Box::new(address),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        let simple_file = "\"file.aya\"";
        let (input, path) = parse_path(simple_file).unwrap();
        assert!(input.is_empty());
        assert_eq!(path, "file.aya");

        let complex_path = "\"./../some_path/some/./../file.aya\"";
        let (input, path) = parse_path(complex_path).unwrap();
        assert!(input.is_empty());
        assert_eq!(path, "./../some_path/some/./../file.aya");
    }

    #[test]
    fn test_parse_import() {
        let input = "import \"./path.aya\" module_name &fefe {}";
        let (input, import) = parse_import(input).unwrap();
        assert!(input.is_empty());
        insta::assert_debug_snapshot!(import);
    }

    #[test]
    fn test_parse_import_with_variables() {
        let input = r#"
            import "./path.aya" module_name &fefe {
                variable_a: $C0D3,
                variable_b: [!other_variable],
                varaible_c: &FEFE,
                variable_d: [other_module.variable],
            }
        "#;
        let (input, import) = parse_import(input).unwrap();
        assert!(input.is_empty());
        insta::assert_debug_snapshot!(import);
    }
}
