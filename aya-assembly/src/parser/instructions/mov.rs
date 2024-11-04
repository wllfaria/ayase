use crate::lexer::{Kind, Lexer};
use crate::parser::ast::{Instruction, Statement};
use crate::parser::common::{expect, parse_hex_lit, parse_keyword, parse_register, parse_variable, peek};
use crate::parser::error::{
    ADDRESS_HELP, ADDRESS_MSG, BRACKETED_EXPR_HELP, BRACKETED_EXPR_MSG, COMMA_MSG, HEX_LIT_HELP, HEX_LIT_MSG, VAR_HELP,
    VAR_MSG,
};
use crate::parser::expressions::{parse_address_expr, parse_literal_expr};
use crate::parser::Result;
use crate::utils::unexpected_token;

pub fn parse_mov<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    parse_keyword(source.as_ref(), lexer, Kind::Mov)?;

    let lhs_token = peek(source.as_ref(), lexer)?;
    let lhs = match lhs_token.kind {
        Kind::Ident => Statement::Register(parse_register(source.as_ref(), lexer)?),
        Kind::Ampersand => parse_address_expr(source.as_ref(), lexer, ADDRESS_HELP, ADDRESS_MSG)?,
        _ => return unexpected_token(source.as_ref(), &lhs_token),
    };

    expect(
        Kind::Comma,
        lexer,
        source.as_ref(),
        "missing a comma after left side of instruction",
        COMMA_MSG,
    )?;

    let rhs_token = peek(source.as_ref(), lexer)?;
    let rhs = match rhs_token.kind {
        Kind::Ident => Statement::Register(parse_register(source.as_ref(), lexer)?),
        Kind::Bang => Statement::Var(parse_variable(source.as_ref(), lexer, VAR_HELP, VAR_MSG)?),
        Kind::HexNumber => Statement::HexLiteral(parse_hex_lit(source.as_ref(), lexer, HEX_LIT_HELP, HEX_LIT_MSG)?),
        Kind::Ampersand => parse_address_expr(source.as_ref(), lexer, ADDRESS_HELP, ADDRESS_MSG)?,
        Kind::LBracket => parse_literal_expr(source.as_ref(), lexer, BRACKETED_EXPR_HELP, BRACKETED_EXPR_MSG)?,
        _ => return unexpected_token(source.as_ref(), &rhs_token),
    };

    match (lhs_token.kind, rhs_token.kind) {
        // MovRegReg
        (Kind::Ident, Kind::Ident) => Ok(Instruction::MovRegReg(lhs, rhs).into()),
        // MovLitReg
        (Kind::Ident, Kind::Bang) => Ok(Instruction::MovLitReg(lhs, rhs).into()),
        (Kind::Ident, Kind::HexNumber) => Ok(Instruction::MovLitReg(lhs, rhs).into()),
        (Kind::Ident, Kind::LBracket) => Ok(Instruction::MovLitReg(lhs, rhs).into()),
        // MovRegMem
        (Kind::Ampersand, Kind::Ident) => Ok(Instruction::MovRegMem(lhs, rhs).into()),
        // MovMemReg
        (Kind::Ident, Kind::Ampersand) => Ok(Instruction::MovMemReg(lhs, rhs).into()),
        // MovLitMem
        (Kind::Ampersand, Kind::LBracket) => Ok(Instruction::MovLitMem(lhs, rhs).into()),
        (Kind::Ampersand, Kind::Bang) => Ok(Instruction::MovLitMem(lhs, rhs).into()),
        (Kind::Ampersand, Kind::HexNumber) => Ok(Instruction::MovLitMem(lhs, rhs).into()),
        // MovRegPtrReg
        (Kind::Ampersand, Kind::Ampersand) if is_reg_address(&rhs) && is_reg_address(&lhs) => {
            Ok(Instruction::MovRegPtrReg(lhs, rhs).into())
        }
        _ => return unexpected_token(source.as_ref(), &rhs_token),
    }
}

fn is_reg_address(result: &Statement) -> bool {
    let Statement::Address(inner) = result else {
        return false;
    };
    matches!(inner.as_ref(), Statement::Register(_))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> Statement {
        let mut lexer = Lexer::new(input);
        parse_mov(input, &mut lexer).unwrap()
    }

    #[test]
    fn test_mov_reg_reg() {
        let input = "mov r1, r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovRegReg(_, _)));
    }

    #[test]
    fn test_mov_lit_reg() {
        let input = "mov r1, $c0d3";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovLitReg(_, _)));
    }

    #[test]
    fn test_mov_lit_reg_var() {
        let input = "mov r1, !var";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovLitReg(_, _)));
    }

    #[test]
    fn test_mov_lit_reg_expr() {
        let input = "mov r1, [$c0d3 + r2]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovLitReg(_, _)));
    }

    #[test]
    fn test_mov_reg_mem() {
        let input = "mov &[$c0d3], r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovRegMem(_, _)));
    }

    #[test]
    fn test_mov_reg_mem_var() {
        let input = "mov &[!var], r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovRegMem(_, _)));
    }

    #[test]
    fn test_mov_reg_mem_expr() {
        let input = "mov &[$c0d3 + r2], r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovRegMem(_, _)));
    }

    #[test]
    fn test_mov_mem_reg() {
        let input = "mov r2, &[$c0d3]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovMemReg(_, _)));
    }

    #[test]
    fn test_mov_mem_reg_var() {
        let input = "mov r2, &[!var]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovMemReg(_, _)));
    }

    #[test]
    fn test_mov_mem_reg_expr() {
        let input = "mov r2, &[$c0d3 + r2]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovMemReg(_, _)));
    }

    #[test]
    fn test_mov_lit_mem() {
        let input = "mov &[$c0d3], $c0d3";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovLitMem(_, _)));
    }

    #[test]
    fn test_mov_lit_mem_var() {
        let input = "mov &[!var], $c0d3";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovLitMem(_, _)));
    }

    #[test]
    fn test_mov_lit_mem_expr() {
        let input = "mov &[$c0d3 + r2], $c0d3";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovLitMem(_, _)));
    }

    #[test]
    fn test_mov_lit_var_mem() {
        let input = "mov &[$c0d3], !var";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovLitMem(_, _)));
    }

    #[test]
    fn test_mov_lit_expr_mem() {
        let input = "mov &[$c0d3], [$c0d3 + r2 + !var]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovLitMem(_, _)));
    }

    #[test]
    fn test_mov_lit_expr_mem_var() {
        let input = "mov &[!var], [$c0d3 + r2 + !var]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovLitMem(_, _)));
    }

    #[test]
    fn test_mov_lit_expr_mem_expr() {
        let input = "mov &[!var + $c0d3 + r2], [$c0d3 + r2 + !var]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovLitMem(_, _)));
    }

    #[test]
    fn test_mov_reg_ptr_reg() {
        let input = "mov &[r2], &[r3]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::MovRegPtrReg(_, _)));
    }
}
