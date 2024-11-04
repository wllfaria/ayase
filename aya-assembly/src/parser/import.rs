use super::common::{expect, parse_hex_lit, parse_identifier, parse_string, parse_variable};
use super::error::{
    Result, ADDRESS_HELP, ADDRESS_MSG, COLON_MSG, COMMA_MSG, DOT_MSG, HEX_LIT_HELP, HEX_LIT_MSG, IDENT_MSG, LBRACE_MSG,
    LBRACKET_MSG, PATH_MSG, RBRACE_MSG, RBRACKET_MSG,
};
use crate::lexer::{Kind, Lexer, TransposeRef};
use crate::parser::ast::Statement;
use crate::parser::syntax::parse_simple_address;
use crate::utils::{unexpected_eof, unexpected_token};

fn parse_field_accessor<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    let module = parse_identifier(
        source.as_ref(),
        lexer,
        "module name in import value must be a valid identifier",
        IDENT_MSG,
    )?;

    expect(
        Kind::Dot,
        lexer,
        source.as_ref(),
        "module field accessor must be dot separated",
        DOT_MSG,
    )?;

    let field = parse_identifier(
        source.as_ref(),
        lexer,
        "module name in import value must be a valid identifier",
        IDENT_MSG,
    )?;

    Ok(Statement::FieldAccessor { module, field })
}

fn parse_bracketed_value<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    expect(
        Kind::LBracket,
        lexer,
        source.as_ref(),
        "bracketed import value must start with a left bracket",
        LBRACKET_MSG,
    )?;

    let Ok(Some(token)) = lexer.peek().transpose() else {
        let Err(err) = lexer.next().transpose() else {
            return unexpected_eof(source.as_ref(), "unterminated import statement");
        };
        return Err(err);
    };

    let statement = match token.kind {
        Kind::Ident => parse_field_accessor(source.as_ref(), lexer)?,
        Kind::Bang => Statement::Var(parse_variable(
            source.as_ref(),
            lexer,
            "variable value must be a valid identifier",
            IDENT_MSG,
        )?),
        _ => return unexpected_token(source.as_ref(), token),
    };

    expect(
        Kind::RBracket,
        lexer,
        source.as_ref(),
        "unfinished bracketed import value",
        RBRACKET_MSG,
    )?;

    Ok(statement)
}

fn parse_import_var_value<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    let Ok(Some(token)) = lexer.peek().transpose() else {
        let Err(err) = lexer.next().transpose() else {
            return unexpected_eof(source.as_ref(), "unterminated import statement");
        };
        return Err(err);
    };

    match token.kind {
        Kind::LBracket => parse_bracketed_value(source.as_ref(), lexer),
        Kind::Bang => Ok(Statement::Var(parse_variable(
            source.as_ref(),
            lexer,
            "variable value must be a valid identifier",
            IDENT_MSG,
        )?)),
        Kind::HexNumber => Ok(Statement::HexLiteral(parse_hex_lit(
            source.as_ref(),
            lexer,
            HEX_LIT_HELP,
            HEX_LIT_MSG,
        )?)),
        _ => return unexpected_token(source.as_ref(), token),
    }
}

fn parse_import_vars<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Vec<Statement>> {
    let mut variables = vec![];

    loop {
        let Ok(Some(token)) = lexer.peek().transpose() else {
            let Err(err) = lexer.next().transpose() else {
                return unexpected_eof(source.as_ref(), "unterminated import statement");
            };
            return Err(err);
        };

        if token.kind == Kind::RBrace {
            break;
        }

        let name = parse_identifier(
            source.as_ref(),
            lexer,
            "variable name must be a valid identifier",
            IDENT_MSG,
        )?;

        expect(
            Kind::Colon,
            lexer,
            source.as_ref(),
            "import variable name and value must be separated by a colon",
            COLON_MSG,
        )?;

        let value = parse_import_var_value(source.as_ref(), lexer)?;

        let Ok(Some(next)) = lexer.peek().transpose() else {
            let Err(err) = lexer.next().transpose() else {
                return unexpected_eof(source.as_ref(), "unterminated import statement");
            };
            return Err(err);
        };

        match next.kind {
            Kind::RBrace => {}
            _ => {
                _ = expect(
                    Kind::Comma,
                    lexer,
                    source.as_ref(),
                    "import variables must be separated by a comma",
                    COMMA_MSG,
                )?
            }
        }

        variables.push(Statement::ImportVar {
            name,
            value: Box::new(value),
        })
    }

    Ok(variables)
}

pub fn parse_import<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    lexer.next().transpose()?;

    let path = parse_string(
        source.as_ref(),
        lexer,
        "path for modules must be in string format",
        PATH_MSG,
    )?;

    let name = parse_identifier(
        source.as_ref(),
        lexer,
        "module name must be a valid identifier",
        IDENT_MSG,
    )?;

    let Statement::Address(address) = parse_simple_address(source.as_ref(), lexer, ADDRESS_HELP, ADDRESS_MSG)? else {
        unreachable!();
    };

    expect(
        Kind::LBrace,
        lexer,
        source.as_ref(),
        "modules must have a variable declaration block",
        LBRACE_MSG,
    )?;

    let variables = parse_import_vars(source.as_ref(), lexer)?;

    expect(
        Kind::RBrace,
        lexer,
        source.as_ref(),
        "unclosed module declaration block. you most likely forgot a `}` [RIGHT_CURLY]",
        RBRACE_MSG,
    )?;

    Ok(Statement::Import {
        name,
        path,
        address,
        variables,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_import_with_variables() {
        let input = r#"
            import "./path.aya" module_name &[$fefe] {
                variable_a: $C0D3,
                variable_b: [!other_variable],
                variable_c: [other_module.variable],
                variable_d: !other_variable,
            }
        "#;
        let result = crate::parser::parse(input).unwrap();
        insta::assert_debug_snapshot!(result);
    }
}
