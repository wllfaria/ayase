use crate::lexer::{Kind, Lexer, TransposeRef};
use crate::parser::ast::{Instruction, Statement};
use crate::parser::common::{parse_hex_lit, parse_keyword, parse_register};
use crate::parser::error::{unexpected_eof, unexpected_token, HEX_LIT_HELP, HEX_LIT_MSG};
use crate::parser::Result;

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
        _ => return unexpected_token(source.as_ref(), token),
    };

    match kind {
        Kind::Ident => Ok(Instruction::PshReg(value).into()),
        Kind::HexNumber => Ok(Instruction::PshLit(value).into()),
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
}
