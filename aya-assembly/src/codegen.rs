use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::PathBuf;

use crate::mod_resolver::{Either, ResolvedModules};
use crate::parser::ast::{Ast, Instruction, Operator, Statement};

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
    temp_registers: Vec<String>,
    used_registers: Vec<String>,
}

impl<'codegen> CodeGenerator<'codegen> {
    fn new(source: &'codegen str, ast: &'codegen Ast) -> Self {
        Self {
            source,
            ast,
            code: Vec::new(),
            temp_registers: vec![
                "acc".into(),
                "r1".into(),
                "r2".into(),
                "r3".into(),
                "r4".into(),
                "r5".into(),
                "r6".into(),
                "r7".into(),
                "r8".into(),
            ],
            used_registers: Vec::with_capacity(8),
        }
    }

    fn get_register(&self, offset: &Statement) -> &str {
        let Statement::Register(offset) = offset else {
            unreachable!();
        };
        &self.source[Range::from(*offset)]
    }

    fn generate(&mut self) -> miette::Result<()> {
        for stat in self.ast.statements.iter() {
            match stat {
                Statement::Data {
                    name,
                    size,
                    exported,
                    values,
                } => {
                    let exported = if *exported { "+" } else { "" };
                    let name = &self.source[Range::from(*name)];
                    let mut values_str = vec![];

                    for value in values {
                        match value {
                            Statement::Address(stat) => match stat.as_ref() {
                                Statement::HexLiteral(offset) => {
                                    let num = &self.source[Range::from(*offset)];
                                    values_str.push(format!("&[${num}]"));
                                }
                                _ => unreachable!(),
                            },
                            Statement::HexLiteral(offset) => {
                                let num = &self.source[Range::from(*offset)];
                                values_str.push(format!("${num}"));
                            }
                            _ => unreachable!(),
                        }
                    }

                    let values = values_str.join(", ");
                    self.code.push(format!("{exported}data{size} {name} = {{ {values} }}"));
                }
                Statement::Label { name, exported } => {
                    let exported = if *exported { "+" } else { "" };
                    let name = &self.source[Range::from(*name)];
                    self.code.push(format!("{exported}{name}:"))
                }
                Statement::Instruction(inst) => match inst.as_ref() {
                    Instruction::MovLitReg(lhs, rhs) => {
                        let lhs = self.get_register(lhs);
                        self.generate_code(rhs, Some(lhs.into()))?;
                        self.release_all_temp_registers();
                    }
                    Instruction::MovRegReg(lhs, rhs) => {
                        let lhs = self.get_register(lhs);
                        let rhs = self.get_register(rhs);
                        self.code.push(format!("mov {lhs}, {rhs}"));
                    }
                    Instruction::MovRegMem(_, _) => todo!(),
                    Instruction::MovMemReg(_, _) => todo!(),
                    Instruction::MovLitMem(_, _) => todo!(),
                    Instruction::MovRegPtrReg(_, _) => todo!(),
                    Instruction::AddRegReg(_, _) => todo!(),
                    Instruction::AddLitReg(_, _) => todo!(),
                    Instruction::SubRegReg(_, _) => todo!(),
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
                    Instruction::JleLit(_, _) => todo!(),
                    Instruction::JleReg(_, _) => todo!(),
                    Instruction::JltLit(_, _) => todo!(),
                    Instruction::JltReg(_, _) => todo!(),
                    Instruction::Jmp(_) => todo!(),
                    Instruction::PshLit(_) => todo!(),
                    Instruction::PshReg(_) => todo!(),
                    Instruction::Pop(_) => todo!(),
                    Instruction::CallLit(_) => todo!(),
                    Instruction::Ret(_) => todo!(),
                    Instruction::Hlt(_) => todo!(),
                },
                _ => {}
            }
        }

        Ok(())
    }

    fn generate_code(&mut self, node: &Statement, target: Option<String>) -> miette::Result<String> {
        if let Some(value) = self.evaluate_constants(node) {
            let dest = target.unwrap_or_else(|| self.get_temp_register().unwrap());
            self.code.push(format!("  mov {dest}, {value}"));
            return Ok(dest);
        };

        match node {
            Statement::HexLiteral(value) => {
                let dest = target.unwrap_or_else(|| self.get_temp_register().unwrap());
                let value = &self.source[Range::from(*value)];
                self.code.push(format!("  mov {dest}, {value}"));
                Ok(dest)
            }
            Statement::Register(reg) => {
                let dest = target.unwrap_or_else(|| self.get_temp_register().unwrap());
                let reg = &self.source[Range::from(*reg)];
                self.code.push(format!("  mov {dest}, {reg}"));
                Ok(dest)
            }
            Statement::BinaryOp { lhs, operator, rhs } => {
                let lhs = self.generate_code(lhs, None)?;
                let rhs = self.generate_code(rhs, None)?;

                // Determine the target register
                let dest = target.unwrap_or(lhs.clone());
                // If dest is not lhs, move lhs to dest
                if dest != lhs {
                    self.code.push(format!("  mov {dest}, {lhs}"));
                }
                self.code.push(format!("  {operator} {dest}, {rhs}"));

                if !self.used_registers.contains(&rhs) {
                    self.release_temp_register(rhs);
                }

                Ok(dest)
            }
            _ => unreachable!(),
        }
    }

    fn get_temp_register(&mut self) -> Result<String, String> {
        if let Some(reg) = self.temp_registers.pop() {
            // Save the original value of the register
            self.code.push(format!("  psh {reg}"));
            self.used_registers.push(reg.clone());
            Ok(reg)
        } else {
            Err("No temporary registers available".to_string())
        }
    }

    fn release_all_temp_registers(&mut self) {
        while let Some(reg) = self.used_registers.pop() {
            self.code.push(format!("  pop {reg}"));
            self.temp_registers.push(reg);
        }
    }

    fn release_temp_register(&mut self, reg: String) {
        self.code.push(format!("pop {reg}"));
        self.used_registers.retain(|r| r != &reg);
        self.temp_registers.push(reg);
    }

    fn evaluate_constants(&self, node: &Statement) -> Option<String> {
        if let Statement::HexLiteral(value) = node {
            let value = &self.source[Range::from(*value)];
            return Some(format!("${value}"));
        };

        if let Statement::BinaryOp { lhs, operator, rhs } = node {
            if let (Some(lhs), Some(rhs)) = (self.evaluate_constants(lhs), self.evaluate_constants(rhs)) {
                let lhs = u16::from_str_radix(&lhs, 16).unwrap();
                let rhs = u16::from_str_radix(&rhs, 16).unwrap();

                let result = match operator {
                    Operator::Add => lhs.wrapping_add(rhs),
                    Operator::Sub => lhs.wrapping_sub(rhs),
                    Operator::Mul => lhs.wrapping_mul(rhs),
                };
                return Some(format!("{result:X}"));
            }
        };

        None
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
    Ok(gen_modules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mod_resolver;

    #[test]
    fn test_codegen() {
        let code = r#"
        data8 data = { $00, $03 }
        data16 data = { $00, &[$03] }

        +label:
            mov r2, [r2 + r3 * ($12 * $12)]
        label_2:
            mov r2, [r2 + r3 * ($12 * $12)]
        "#;
        let modules = mod_resolver::resolve(code.into(), "path.aya").unwrap();
        let modules = generate(modules).unwrap();
        println!("{}", modules[0].code);

        panic!();
    }
}
