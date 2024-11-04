use crate::lexer::{Kind, Lexer, TransposeRef};
use crate::parser::ast::{Instruction, Statement};
use crate::parser::common::{parse_hex_lit, parse_keyword, parse_register, parse_variable};
use crate::parser::error::{BRACKETED_EXPR_HELP, BRACKETED_EXPR_MSG, HEX_LIT_HELP, HEX_LIT_MSG, VAR_HELP, VAR_MSG};
use crate::parser::expressions::parse_literal_expr;
use crate::parser::Result;
use crate::utils::{unexpected_eof, unexpected_token};

pub fn parse_psh<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    parse_keyword(source.as_ref(), lexer, Kind::Psh)?;

    let Ok(Some(token)) = lexer.peek().transpose() else {
        let Err(err) = lexer.next().transpose() else {
            return unexpected_eof(source.as_ref(), "unterminated import statement");
        };
        return Err(err);
    };
    let kind = token.kind;

    let value = match kind {
        Kind::Ident => Statement::Register(parse_register(source.as_ref(), lexer)?),
        Kind::HexNumber => Statement::HexLiteral(parse_hex_lit(source.as_ref(), lexer, HEX_LIT_HELP, HEX_LIT_MSG)?),
        Kind::Bang => Statement::Var(parse_variable(source.as_ref(), lexer, VAR_HELP, VAR_MSG)?),
        Kind::LBracket => parse_literal_expr(source.as_ref(), lexer, BRACKETED_EXPR_HELP, BRACKETED_EXPR_MSG)?,
        _ => return unexpected_token(source.as_ref(), token),
    };

    match kind {
        Kind::Ident => Ok(Instruction::PshReg(value).into()),
        Kind::HexNumber => Ok(Instruction::PshLit(value).into()),
        Kind::Bang => Ok(Instruction::PshLit(value).into()),
        Kind::LBracket => Ok(Instruction::PshLit(value).into()),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> Statement {
        let mut lexer = Lexer::new(input);
        parse_psh(input, &mut lexer).unwrap()
    }

    #[test]
    fn test_psh_reg() {
        let input = "psh r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_psh_lit() {
        let input = "psh $0303";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_psh_lit_var() {
        let input = "psh !var";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_psh_lit_expr() {
        let input = "psh [$0303 + r2]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);
    }
}
