use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;

use aya_cpu::register::Register;

use crate::mod_resolver::{Either, ResolvedModules};
use crate::parser::ast::{Ast, Instruction, Operator, Statement};
use crate::parser::error::{REGISTER_HELP, REGISTER_MSG};
use crate::utils::{bail, unexpected_statement};

macro_rules! formatted {
    ($prefix:ident, $lhs:ident, $rhs:ident) => {
        format!("{} {}, {}", $prefix, $lhs, $rhs)
    };
    ($prefix:ident, $lhs:expr, $rhs:ident) => {
        format!("{} {}, {}", $prefix, format_args!($lhs), $rhs)
    };
    ($prefix:ident, $lhs:ident, $rhs:expr) => {
        format!("{} {}, {}", $prefix, $lhs, format_args!($rhs))
    };
    ($prefix:ident, $lhs:expr, $rhs:expr) => {
        format!("{} {}, {}", $prefix, format_args!($lhs), format_args!($rhs))
    };
    ($prefix:ident, $lhs:ident) => {
        format!("{} {}", $prefix, $lhs)
    };
    ($prefix:ident, $lhs:expr) => {
        format!("{} {}", $prefix, format_args!($lhs))
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum InstructionPrefix {
    Mov,
    Add,
    Sub,
    Mul,
    Inc,
    Dec,
    Lsh,
    Rhs,
    And,
    Or,
    Xor,
    Not,
    Psh,
    Pop,
    Call,
    Ret,
    Jeq,
    Jgt,
    Jne,
    Jge,
    Jle,
    Jlt,
    Jmp,
    Hlt,
}

impl std::fmt::Display for InstructionPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionPrefix::Mov => write!(f, "MOV"),
            InstructionPrefix::Add => write!(f, "ADD"),
            InstructionPrefix::Sub => write!(f, "SUB"),
            InstructionPrefix::Mul => write!(f, "MUL"),
            InstructionPrefix::Inc => write!(f, "INC"),
            InstructionPrefix::Dec => write!(f, "DEC"),
            InstructionPrefix::Lsh => write!(f, "LSH"),
            InstructionPrefix::Rhs => write!(f, "RHS"),
            InstructionPrefix::And => write!(f, "AND"),
            InstructionPrefix::Or => write!(f, "OR"),
            InstructionPrefix::Xor => write!(f, "XOR"),
            InstructionPrefix::Not => write!(f, "NOT"),
            InstructionPrefix::Psh => write!(f, "PSH"),
            InstructionPrefix::Pop => write!(f, "POP"),
            InstructionPrefix::Call => write!(f, "CALL"),
            InstructionPrefix::Ret => write!(f, "RET"),
            InstructionPrefix::Jeq => write!(f, "JEQ"),
            InstructionPrefix::Jgt => write!(f, "JGT"),
            InstructionPrefix::Jne => write!(f, "JNE"),
            InstructionPrefix::Jge => write!(f, "JGE"),
            InstructionPrefix::Jle => write!(f, "JLE"),
            InstructionPrefix::Jlt => write!(f, "JLT"),
            InstructionPrefix::Jmp => write!(f, "JMP"),
            InstructionPrefix::Hlt => write!(f, "HLT"),
        }
    }
}

#[derive(Debug)]
pub struct CodegenModule {
    pub name: String,
    pub code: String,
    pub path: PathBuf,
    pub imports: Vec<PathBuf>,
    pub symbols: HashMap<String, u16>,
    pub variables: Option<HashMap<String, Either>>,
    pub exports: HashMap<String, u16>,
}

#[derive(Debug)]
pub struct CodeGenerator<'codegen> {
    source: &'codegen str,
    ast: &'codegen Ast,
    code: Vec<String>,
    temp_registers: Vec<Register>,
    used_registers: Vec<Register>,
}

trait ToExportedPrefix {
    fn to_exported_prefix(&self) -> &str;
}

impl ToExportedPrefix for bool {
    fn to_exported_prefix(&self) -> &str {
        if *self {
            "+"
        } else {
            ""
        }
    }
}

impl<'codegen> CodeGenerator<'codegen> {
    fn new(source: &'codegen str, ast: &'codegen Ast) -> Self {
        Self {
            source,
            ast,
            code: Vec::new(),
            temp_registers: vec![
                Register::Acc,
                Register::R1,
                Register::R2,
                Register::R3,
                Register::R4,
                Register::R5,
                Register::R6,
                Register::R7,
                Register::R8,
            ],
            used_registers: Vec::with_capacity(8),
        }
    }

    fn generate(&mut self) -> miette::Result<()> {
        for stat in self.ast.statements.iter() {
            match stat {
                Statement::Data { .. } => self.gen_data(stat)?,
                Statement::Label { .. } => self.gen_label(stat),
                Statement::Instruction(inst) => self.gen_instruction(inst.as_ref())?,
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    fn generate_code(
        &mut self,
        prefix: InstructionPrefix,
        node: &Statement,
        target: Option<Register>,
    ) -> miette::Result<Register> {
        if let Some(value) = self.evaluate_constants(node)? {
            let dest = target.unwrap_or_else(|| self.get_temp_register().unwrap());
            self.code.push(formatted!(prefix, dest, value));
            return Ok(dest);
        };

        match node {
            Statement::HexLiteral(value) => {
                let dest = target.unwrap_or_else(|| self.get_temp_register().unwrap());
                let value = &self.source[Range::from(*value)];
                self.code.push(formatted!(prefix, dest, value));
                Ok(dest)
            }
            Statement::Register(reg) => {
                let dest = target.unwrap_or_else(|| self.get_temp_register().unwrap());
                let reg = &self.source[Range::from(*reg)];
                self.code.push(formatted!(prefix, dest, reg));
                Ok(dest)
            }
            Statement::BinaryOp { lhs, operator, rhs } => {
                let lhs = self.generate_code(InstructionPrefix::Mov, lhs, None)?;
                let rhs = self.generate_code(InstructionPrefix::Mov, rhs, None)?;

                // Determine the target register
                let dest = target.unwrap_or(lhs);
                // If dest is not lhs, move lhs to dest
                if dest != lhs {
                    let prefix = InstructionPrefix::Mov;
                    self.code.push(formatted!(prefix, dest, lhs));
                }
                self.code.push(formatted!(operator, dest, rhs));

                if !self.used_registers.contains(&rhs) {
                    self.release_temp_register(rhs);
                }

                Ok(dest)
            }
            _ => unreachable!(),
        }
    }

    fn get_register(&self, offset: &Statement) -> miette::Result<Register> {
        let Statement::Register(offset) = offset else {
            unreachable!();
        };
        let reg_name = &self.source[Range::from(*offset)];
        match Register::try_from(reg_name) {
            Ok(reg) => Ok(reg),
            Err(_) => Err(bail(self.source, REGISTER_HELP, REGISTER_MSG, *offset)),
        }
    }

    fn get_address(&self, node: &Statement) -> miette::Result<String> {
        let Statement::Address(inner) = node else {
            return unexpected_statement(
                self.source,
                "unexpected statement, expected: [HEX_LITERAL]",
                node.offset(),
            );
        };
        let value = &self.source[Range::from(inner.offset())];
        match inner.as_ref() {
            Statement::Register(_) => Ok(value.to_string()),
            Statement::HexLiteral(_) => self.gen_hex_lit(inner.as_ref()),
            Statement::Var(_) => self.gen_var(inner.as_ref()),
            stat => unexpected_statement(
                self.source,
                "unexpected statement, expected: [HEX_LITERAL]",
                stat.offset(),
            ),
        }
    }

    fn get_temp_register(&mut self) -> miette::Result<Register> {
        if let Some(reg) = self.temp_registers.pop() {
            let prefix = InstructionPrefix::Psh;
            self.code.push(formatted!(prefix, reg));
            self.used_registers.push(reg);
            Ok(reg)
        } else {
            panic!();
        }
    }

    fn release_all_temp_registers(&mut self) {
        while let Some(reg) = self.used_registers.pop() {
            let prefix = InstructionPrefix::Pop;
            self.code.push(formatted!(prefix, reg));
            self.temp_registers.push(reg);
        }
    }

    fn release_temp_register(&mut self, reg: Register) {
        let prefix = InstructionPrefix::Pop;
        self.code.push(formatted!(prefix, reg));
        self.used_registers.retain(|r| *r != reg);
        self.temp_registers.push(reg);
    }

    fn evaluate_constants(&self, node: &Statement) -> miette::Result<Option<String>> {
        if let Statement::HexLiteral(value) = node {
            return Ok(Some(self.gen_hex_lit(node)?));
        };

        if let Statement::BinaryOp { lhs, operator, rhs } = node {
            if let (Some(lhs_str), Some(rhs_str)) = (self.evaluate_constants(lhs)?, self.evaluate_constants(rhs)?) {
                let Ok(lhs) = u16::from_str_radix(&lhs_str, 16) else {
                    return Err(bail(
                        self.source,
                        "[INVALID_STATEMENT]: error while compiling statement",
                        "hex number is not within the u16 range",
                        lhs.offset(),
                    ));
                };
                let Ok(rhs) = u16::from_str_radix(&rhs_str, 16) else {
                    return Err(bail(
                        self.source,
                        "[INVALID_STATEMENT]: error while compiling statement",
                        "hex number is not within the u16 range",
                        rhs.offset(),
                    ));
                };

                let result = match operator {
                    Operator::Add => lhs.wrapping_add(rhs),
                    Operator::Sub => lhs.wrapping_sub(rhs),
                    Operator::Mul => lhs.wrapping_mul(rhs),
                };
                return Ok(Some(format!("{result:X}")));
            }
        };

        Ok(None)
    }

    fn gen_hex_lit(&self, statement: &Statement) -> miette::Result<String> {
        match statement {
            Statement::HexLiteral(offset) => {
                let num = &self.source[Range::from(*offset)];
                Ok(format!("${}", num.to_uppercase()))
            }
            _ => Err(bail(
                self.source,
                "unexpected statement, expected: [HEX_LITERAL]",
                "[SYNTAX_ERROR]: unexpected statement",
                statement.offset(),
            )),
        }
    }

    fn gen_var(&self, statement: &Statement) -> miette::Result<String> {
        match statement {
            Statement::Var(offset) => {
                let var = &self.source[Range::from(*offset)];
                Ok(format!("!{var}"))
            }
            _ => Err(bail(
                self.source,
                "unexpected statement, expected: [VAR]",
                "[SYNTAX_ERROR]: unexpected statement",
                statement.offset(),
            )),
        }
    }

    fn gen_data(&mut self, statement: &Statement) -> miette::Result<()> {
        let Statement::Data {
            name,
            size,
            exported,
            values,
        } = statement
        else {
            unreachable!()
        };
        let exported = exported.to_exported_prefix();
        let name = &self.source[Range::from(*name)];

        let mut values_str = vec![];
        for value in values {
            match value {
                Statement::Address(stat) => values_str.push(format!("&[{}]", self.gen_hex_lit(stat.as_ref())?)),
                Statement::HexLiteral(_) => values_str.push(self.gen_hex_lit(value)?),
                _ => {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [HEX_LITERAL]",
                        value.offset(),
                    )
                }
            }
        }

        let values = values_str.join(", ");
        self.code.push(format!("{exported}data{size} {name} = {{ {values} }}"));
        Ok(())
    }

    fn gen_label(&mut self, statement: &Statement) {
        let Statement::Label { name, exported } = statement else { unreachable!() };
        let exported = exported.to_exported_prefix();
        let name = &self.source[Range::from(*name)];
        self.code.push(format!("{exported}{name}:"));
    }

    fn gen_instruction(&mut self, instruction: &Instruction) -> miette::Result<()> {
        match instruction {
            Instruction::MovLitReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Mov;
                let lhs = self.get_register(lhs)?;

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, lhs, "!{var_name}"));
                    return Ok(());
                }

                self.generate_code(prefix, rhs, Some(lhs))?;
                self.release_all_temp_registers();
            }
            Instruction::MovRegReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Mov;
                let lhs = lhs.offset().get_source(&self.source);
                let rhs = rhs.offset().get_source(&self.source);
                self.code.push(formatted!(prefix, lhs, rhs));
            }
            Instruction::MovRegMem(lhs, rhs) => {
                let prefix = InstructionPrefix::Mov;
                let lhs = self.get_address(lhs)?;
                let rhs = rhs.offset().get_source(&self.source);
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
            }
            Instruction::MovMemReg(_, _) => todo!(),
            Instruction::MovLitMem(lhs, rhs) => {}
            Instruction::MovRegPtrReg(_, _) => todo!(),
            Instruction::AddRegReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Sub;
                let lhs = lhs.offset().get_source(&self.source);
                let rhs = rhs.offset().get_source(&self.source);
                self.code.push(formatted!(prefix, lhs, rhs));
            }
            Instruction::AddLitReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Add;
                let lhs = self.get_register(lhs)?;
                let rhs = self.generate_code(InstructionPrefix::Mov, rhs, None)?;
                self.code.push(formatted!(prefix, lhs, rhs));
                self.release_all_temp_registers();
            }
            Instruction::SubRegReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Sub;
                let lhs = lhs.offset().get_source(&self.source);
                let rhs = rhs.offset().get_source(&self.source);
                self.code.push(formatted!(prefix, lhs, rhs));
            }
            Instruction::SubLitReg(_, _) => todo!(),
            Instruction::MulRegReg(_, _) => todo!(),
            Instruction::MulLitReg(_, _) => todo!(),
            Instruction::LshRegReg(_, _) => todo!(),
            Instruction::LshLitReg(_, _) => todo!(),
            Instruction::RshRegReg(_, _) => todo!(),
            Instruction::RshLitReg(_, _) => todo!(),
            Instruction::AndRegReg(_, _) => todo!(),
            Instruction::AndLitReg(_, _) => todo!(),
            Instruction::OrLitReg(_, _) => todo!(),
            Instruction::OrRegReg(_, _) => todo!(),
            Instruction::XorLitReg(_, _) => todo!(),
            Instruction::XorRegReg(_, _) => todo!(),
            Instruction::Inc(_) => todo!(),
            Instruction::Dec(_) => todo!(),
            Instruction::Not(_) => todo!(),
            Instruction::JeqLit(_, _) => todo!(),
            Instruction::JeqReg(_, _) => todo!(),
            Instruction::JgtLit(_, _) => todo!(),
            Instruction::JgtReg(_, _) => todo!(),
            Instruction::JneLit(_, _) => todo!(),
            Instruction::JneReg(_, _) => todo!(),
            Instruction::JgeLit(_, _) => todo!(),
            Instruction::JgeReg(_, _) => todo!(),
            Instruction::JleLit(lhs, rhs) => {
                let prefix = InstructionPrefix::Jle;
                let lhs = self.get_address(lhs)?;
                let dest = self.generate_code(InstructionPrefix::Mov, rhs, None)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", dest));
                self.release_all_temp_registers();
            }
            Instruction::JleReg(_, _) => todo!(),
            Instruction::JltLit(_, _) => todo!(),
            Instruction::JltReg(_, _) => todo!(),
            Instruction::Jmp(address) => {
                let prefix = InstructionPrefix::Jmp;
                let address = self.get_address(address)?;
                self.code.push(formatted!(prefix, "&[{address}]"));
            }
            Instruction::PshLit(_) => todo!(),
            Instruction::PshReg(reg) => {
                let prefix = InstructionPrefix::Psh;
                let reg = self.get_register(reg)?;
                self.code.push(formatted!(prefix, reg));
            }
            Instruction::Pop(reg) => {
                let prefix = InstructionPrefix::Pop;
                let reg = reg.offset().get_source(&self.source);
                self.code.push(formatted!(prefix, reg));
            }
            Instruction::CallLit(lit) => {
                let prefix = InstructionPrefix::Call;
                let dest = self.generate_code(InstructionPrefix::Mov, lit, None)?;
                self.code.push(formatted!(prefix, dest));
                self.release_all_temp_registers();
            }
            Instruction::Ret(_) => self.code.push("ret".to_string()),
            Instruction::Hlt(_) => self.code.push("hlt".to_string()),
        };

        Ok(())
    }
}

impl std::fmt::Display for CodeGenerator<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code.join("\n"))
    }
}

pub fn generate(modules: ResolvedModules) -> miette::Result<Vec<CodegenModule>> {
    let mut gen_modules = vec![];
    for (module, source, ast) in modules {
        let mut codegen = CodeGenerator::new(&source, &ast);
        codegen.generate()?;
        let code = codegen.to_string();

        let module = CodegenModule {
            code,
            name: module.name,
            path: module.path,
            imports: module.imports,
            symbols: module.symbols,
            variables: module.variables,
            exports: Default::default(),
        };
        gen_modules.push(module);
    }

    for module in gen_modules.iter() {
        println!("{}", module.code)
    }

    Ok(gen_modules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_label() {
        let source = "label:";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, source);

        let source = "+label:";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, source);
    }

    #[test]
    fn test_gen_data() {
        let source = "data8 sample_data = { $0000, $1234, $C0D3 }";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, source);

        let source = "data16 sample_data = { $0000, $1234, $C0D3 }";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, source);

        let source = "+data8 sample_data = { $0000, $1234, $C0D3 }";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, source);

        let source = "+data16 sample_data = { $0000, $1234, $C0D3 }";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, source);
    }

    #[test]
    fn test_gen_mov_lit_reg() {
        let source = "mov r1, $c0d3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "MOV R1, $C0D3");

        let source = "mov r1, !var";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "MOV R1, !var");

        let source = "mov r1, !var";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "MOV R1, !var");

        let source = "mov r1, [$c0d3 + r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, r2
MOV R1, R8
add R1, R7
POP R7
POP R8"#
        );
    }
}
