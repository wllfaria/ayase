use crate::lexer::{Kind, Lexer, TransposeRef};
use crate::parser::ast::{Instruction, Statement};
use crate::parser::common::{parse_hex_lit, parse_keyword};
use crate::parser::error::{HEX_LIT_HELP, HEX_LIT_MSG};
use crate::parser::expressions::parse_address_expr;
use crate::parser::Result;
use crate::utils::{unexpected_eof, unexpected_token};

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
        Kind::HexNumber => Statement::Address(Box::new(Statement::HexLiteral(parse_hex_lit(
            source.as_ref(),
            lexer,
            HEX_LIT_HELP,
            HEX_LIT_MSG,
        )?))),
        Kind::Ampersand => parse_address_expr(source.as_ref(), lexer, HEX_LIT_HELP, HEX_LIT_MSG)?,
        _ => return unexpected_token(source.as_ref(), token),
    };

    match kind {
        Kind::HexNumber => Ok(Instruction::CallLit(value).into()),
        Kind::Ampersand => Ok(Instruction::CallLit(value).into()),
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
    fn test_call_address_expr() {
        let input = "call &[$0303 + r2]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_call_address_expr_var() {
        let input = "call &[!var + r2]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }
}
