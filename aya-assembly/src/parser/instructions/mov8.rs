use crate::lexer::{Kind, Lexer};
use crate::parser::ast::{Instruction, Statement};
use crate::parser::common::{expect, parse_hex_lit, parse_keyword, parse_register, parse_variable, peek};
use crate::parser::error::{ADDRESS_HELP, ADDRESS_MSG, COMMA_MSG, HEX_LIT_HELP, HEX_LIT_MSG, VAR_HELP, VAR_MSG};
use crate::parser::expressions::parse_address_expr;
use crate::parser::{parse_address_var, Result};
use crate::utils::unexpected_token;

pub fn parse_mov8<S: AsRef<str>>(source: S, lexer: &mut Lexer) -> Result<Statement> {
    parse_keyword(source.as_ref(), lexer, Kind::Mov8)?;

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
        Kind::Ampersand => parse_address_var(source.as_ref(), lexer, ADDRESS_HELP, ADDRESS_MSG)?,
        _ => return unexpected_token(source.as_ref(), &rhs_token),
    };

    match (lhs_token.kind, rhs_token.kind) {
        // MovRegReg
        (Kind::Ident, Kind::Ident) => Ok(Instruction::Mov8RegReg(lhs, rhs).into()),
        // MovLitReg
        (Kind::Ident, Kind::Bang) => Ok(Instruction::Mov8LitReg(lhs, rhs).into()),
        (Kind::Ident, Kind::HexNumber) => Ok(Instruction::Mov8LitReg(lhs, rhs).into()),
        (Kind::Ident, Kind::LBracket) => Ok(Instruction::Mov8LitReg(lhs, rhs).into()),
        // MovRegMem
        (Kind::Ampersand, Kind::Ident) => Ok(Instruction::Mov8RegMem(lhs, rhs).into()),
        // MovMemReg
        (Kind::Ident, Kind::Ampersand) => Ok(Instruction::Mov8MemReg(lhs, rhs).into()),
        // MovLitMem
        (Kind::Ampersand, Kind::Bang) => Ok(Instruction::Mov8LitMem(lhs, rhs).into()),
        (Kind::Ampersand, Kind::HexNumber) => Ok(Instruction::Mov8LitMem(lhs, rhs).into()),
        _ => return unexpected_token(source.as_ref(), &rhs_token),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(input: &str) -> Statement {
        let mut lexer = Lexer::new(input);
        parse_mov8(input, &mut lexer).unwrap()
    }

    #[test]
    fn test_mov_reg_reg() {
        let input = "mov8 r1, r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::Mov8RegReg(_, _)));
    }

    #[test]
    fn test_mov_lit_reg() {
        let input = "mov8 r1, $c0";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::Mov8LitReg(_, _)));
    }

    #[test]
    fn test_mov_lit_reg_var() {
        let input = "mov8 r1, !var";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::Mov8LitReg(_, _)));
    }

    #[test]
    fn test_mov_reg_mem() {
        let input = "mov8 &[$c0d3], r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::Mov8RegMem(_, _)));
    }

    #[test]
    fn test_mov_reg_mem_var() {
        let input = "mov8 &[!var], r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::Mov8RegMem(_, _)));
    }

    #[test]
    fn test_mov_reg_mem_expr() {
        let input = "mov8 &[$c0d3 + r2], r2";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::Mov8RegMem(_, _)));
    }

    #[test]
    fn test_mov_mem_reg() {
        let input = "mov8 r2, &[$c0d3]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::Mov8MemReg(_, _)));
    }

    #[test]
    fn test_mov_mem_reg_var() {
        let input = "mov8 r2, &[!var]";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::Mov8MemReg(_, _)));
    }

    #[test]
    fn test_mov_lit_mem() {
        let input = "mov8 &[$c0d3], $c0";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::Mov8LitMem(_, _)));
    }

    #[test]
    fn test_mov_lit_mem_var() {
        let input = "mov8 &[!var], $c0";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::Mov8LitMem(_, _)));
    }

    #[test]
    fn test_mov_lit_mem_expr() {
        let input = "mov8 &[$c0d3 + r2], $c0";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::Mov8LitMem(_, _)));
    }

    #[test]
    fn test_mov_lit_var_mem() {
        let input = "mov8 &[$c0d3], !var";
        let result = run_instruction(input);
        insta::assert_debug_snapshot!(result);

        let Statement::Instruction(inner) = result else {
            unreachable!();
        };
        assert!(matches!(inner.as_ref(), Instruction::Mov8LitMem(_, _)));
    }

    #[test]
    #[should_panic]
    fn test_mov_lit_reg_expr() {
        let input = "mov8 r1, [$c0d3 + r2]";
        run_instruction(input);
    }

    #[test]
    #[should_panic]
    fn test_mov_mem_reg_expr() {
        let input = "mov8 r2, &[$c0d3 + r2]";
        run_instruction(input);
    }

    #[test]
    #[should_panic]
    fn test_mov_lit_expr_mem() {
        let input = "mov8 &[$c0d3], [$c0d3 + r2 + !var]";
        run_instruction(input);
    }

    #[test]
    #[should_panic]
    fn test_mov_lit_expr_mem_var() {
        let input = "mov8 &[!var], [$c0d3 + r2 + !var]";
        run_instruction(input);
    }

    #[test]
    #[should_panic]
    fn test_mov_lit_expr_mem_expr() {
        let input = "mov8 &[!var + $c0d3 + r2], [$c0d3 + r2 + !var]";
        run_instruction(input);
    }

    #[test]
    #[should_panic]
    fn test_mov_reg_ptr_reg() {
        let input = "mov8 &[r2], &[r3]";
        run_instruction(input);
    }
}
