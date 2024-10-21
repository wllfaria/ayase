use crate::ast::{Instruction, Statement};
use crate::lexer::{Kind, Lexer, TransposeRef};
use crate::parser::common::{parse_hex_lit, parse_keyword, parse_variable};
use crate::parser::error::{unexpected_eof, unexpected_token, HEX_LIT_HELP, HEX_LIT_MSG, IDENT_MSG};
use crate::parser::syntax::parse_simple_address;
use crate::parser::Result;

pub fn parse_call<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    parse_keyword(source.as_ref(), lexer, Kind::Call)?;

    let Ok(Some(token)) = lexer.peek().transpose() else {
        let Err(err) = lexer.next().transpose() else {
            return unexpected_eof(source.as_ref(), "unterminated import statement");
        };
        return Err(err);
    };
    let kind = token.kind;

    let value = match kind {
        Kind::HexNumber => Statement::Address(parse_hex_lit(source.as_ref(), lexer, HEX_LIT_HELP, HEX_LIT_MSG)?),
        Kind::Ampersand => parse_simple_address(source.as_ref(), lexer, HEX_LIT_HELP, HEX_LIT_MSG)?,
        Kind::Bang => Statement::Var(parse_variable(
            source.as_ref(),
            lexer,
            "variable must be a valid identifier",
            IDENT_MSG,
        )?),
        _ => return unexpected_token(source.as_ref(), token),
    };

    match kind {
        Kind::HexNumber => Ok(Instruction::CallLit(value).into()),
        Kind::Ampersand => Ok(Instruction::CallLit(value).into()),
        Kind::Bang => Ok(Instruction::CallLit(value).into()),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> Statement {
        let mut lexer = Lexer::new(input);
        parse_call(input, &mut lexer).unwrap()
    }

    #[test]
    fn test_call_address() {
        let input = "call &[$0303]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_call_lit() {
        let input = "call $0303";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }
}
