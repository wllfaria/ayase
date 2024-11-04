use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;

use aya_cpu::register::Register;

use crate::mod_resolver::{Either, ResolvedModule, ResolvedModules};
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
    Rsh,
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
            InstructionPrefix::Rsh => write!(f, "RSH"),
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
            code: vec![],
            temp_registers: vec![Register::Acc, Register::R5, Register::R6, Register::R7, Register::R8],
            used_registers: Vec::with_capacity(8),
        }
    }

    fn with_module(self, module: &ResolvedModule) -> Self {
        let file = format!("; {} @ {}", module.name, module.path.to_string_lossy());
        Self {
            source: self.source,
            ast: self.ast,
            code: vec![file],
            temp_registers: self.temp_registers,
            used_registers: self.used_registers,
        }
    }

    fn generate(&mut self) -> miette::Result<()> {
        for stat in self.ast.statements.iter() {
            match stat {
                Statement::Data { .. } => self.gen_data(stat)?,
                Statement::Label { .. } => self.gen_label(stat),
                Statement::Const { .. } => self.gen_const(stat)?,
                Statement::Instruction(inst) => self.gen_instruction(inst.as_ref())?,
                _ => {}
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
            let dest = match target {
                Some(target) => target,
                None => self.get_temp_register(node)?,
            };
            self.code.push(formatted!(prefix, dest, value));
            return Ok(dest);
        };

        match node {
            Statement::HexLiteral(value) => {
                let dest = match target {
                    Some(target) => target,
                    None => self.get_temp_register(node)?,
                };
                let value = &self.source[Range::from(*value)];
                self.code.push(formatted!(prefix, dest, value));
                Ok(dest)
            }
            Statement::Register(reg) => {
                let dest = match target {
                    Some(target) => target,
                    None => self.get_temp_register(node)?,
                };
                let reg = &self.source[Range::from(*reg)];
                let reg = match Register::try_from(reg) {
                    Ok(reg) => reg,
                    Err(_) => return Err(bail(self.source, REGISTER_HELP, REGISTER_MSG, node.offset())),
                };
                self.code.push(formatted!(prefix, dest, reg));
                Ok(dest)
            }
            Statement::Var(var) => {
                let dest = match target {
                    Some(target) => target,
                    None => self.get_temp_register(node)?,
                };
                let var = var.get_source(&self.source);
                self.code.push(formatted!(prefix, dest, "!{var}"));
                Ok(dest)
            }
            Statement::BinaryOp { lhs, operator, rhs } => {
                let lhs = self.generate_code(InstructionPrefix::Mov, lhs, None)?;
                let rhs = self.generate_code(InstructionPrefix::Mov, rhs, None)?;
                self.code.push(formatted!(operator, lhs, rhs));

                let dest = target.unwrap_or(lhs);
                if dest != lhs {
                    self.code.push(formatted!(prefix, dest, lhs));
                }

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

    fn get_temp_register(&mut self, node: &Statement) -> miette::Result<Register> {
        if let Some(reg) = self.temp_registers.pop() {
            let prefix = InstructionPrefix::Psh;
            self.code.push(formatted!(prefix, reg));
            self.used_registers.push(reg);
            return Ok(reg);
        };

        Err(bail(
            self.source,
            "this expression is too large, consider decomposing it into multiple instructions",
            "[CODEGEN_ERROR]: expression too large",
            node.offset(),
        ))
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
        if let Statement::HexLiteral(_) = node {
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

    fn gen_const(&mut self, statement: &Statement) -> miette::Result<()> {
        let Statement::Const { name, exported, value } = statement else { unreachable!() };
        let exported = exported.to_exported_prefix();
        let name = &self.source[Range::from(*name)];
        let value = self.gen_hex_lit(value.as_ref())?;
        self.code.push(format!("{exported}const {name} = {value}"));
        Ok(())
    }

    fn gen_instruction(&mut self, instruction: &Instruction) -> miette::Result<()> {
        match instruction {
            Instruction::MovRegReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Mov;
                let lhs = self.get_register(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, lhs, rhs));
            }
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
            Instruction::MovRegMem(lhs, rhs) => {
                let prefix = InstructionPrefix::Mov;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                if let Statement::BinaryOp { .. } = inner.as_ref() {
                    let lhs = self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?;
                    let rhs = self.get_register(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
                    self.release_all_temp_registers();
                    return Ok(());
                }

                let lhs = self.get_address(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
            }
            Instruction::MovMemReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Mov;
                let lhs = self.get_register(lhs)?;

                let Statement::Address(inner) = rhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        rhs.offset(),
                    );
                };

                if let Statement::BinaryOp { .. } = inner.as_ref() {
                    let rhs = self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?;
                    self.code.push(formatted!(prefix, lhs, "&[{rhs}]"));
                    self.release_all_temp_registers();
                    return Ok(());
                }

                let rhs = self.get_address(rhs)?;
                self.code.push(formatted!(prefix, lhs, "&[{rhs}]"));
            }
            Instruction::MovLitMem(lhs, rhs) => {
                let prefix = InstructionPrefix::Mov;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                let lhs = if let Statement::BinaryOp { .. } = inner.as_ref() {
                    self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?
                        .to_string()
                } else {
                    self.get_address(lhs)?
                };

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, "&[{lhs}]", "!{var_name}"));
                    return Ok(());
                }

                if let Statement::HexLiteral(_) = rhs {
                    let hex = self.gen_hex_lit(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", hex));
                    return Ok(());
                }

                let rhs = self.generate_code(InstructionPrefix::Mov, rhs, None)?;
                self.release_all_temp_registers();
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
            }
            Instruction::MovRegPtrReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Mov;
                let lhs = self.get_address(lhs)?;
                let rhs = self.get_address(rhs)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", "&[{rhs}]"));
            }
            Instruction::Inc(reg) => {
                let prefix = InstructionPrefix::Inc;
                let reg = self.get_register(reg)?;
                self.code.push(formatted!(prefix, reg));
            }
            Instruction::Dec(reg) => {
                let prefix = InstructionPrefix::Dec;
                let reg = self.get_register(reg)?;
                self.code.push(formatted!(prefix, reg));
            }
            Instruction::AddRegReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Add;
                let lhs = self.get_register(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, lhs, rhs));
            }
            Instruction::AddLitReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Add;
                let lhs = self.get_register(lhs)?;

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, lhs, "!{var_name}"));
                    return Ok(());
                }

                self.generate_code(prefix, rhs, Some(lhs))?;
                self.release_all_temp_registers();
            }
            Instruction::SubRegReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Sub;
                let lhs = self.get_register(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, lhs, rhs));
            }
            Instruction::SubLitReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Sub;
                let lhs = self.get_register(lhs)?;

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, lhs, "!{var_name}"));
                    return Ok(());
                }

                self.generate_code(prefix, rhs, Some(lhs))?;
                self.release_all_temp_registers();
            }
            Instruction::MulRegReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Mul;
                let lhs = self.get_register(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, lhs, rhs));
            }
            Instruction::MulLitReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Mul;
                let lhs = self.get_register(lhs)?;

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, lhs, "!{var_name}"));
                    return Ok(());
                }

                self.generate_code(prefix, rhs, Some(lhs))?;
                self.release_all_temp_registers();
            }
            Instruction::LshRegReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Lsh;
                let lhs = self.get_register(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, lhs, rhs));
            }
            Instruction::LshLitReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Lsh;
                let lhs = self.get_register(lhs)?;

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, lhs, "!{var_name}"));
                    return Ok(());
                }

                self.generate_code(prefix, rhs, Some(lhs))?;
                self.release_all_temp_registers();
            }
            Instruction::RshRegReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Rsh;
                let lhs = self.get_register(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, lhs, rhs));
            }
            Instruction::RshLitReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Rsh;
                let lhs = self.get_register(lhs)?;

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, lhs, "!{var_name}"));
                    return Ok(());
                }

                self.generate_code(prefix, rhs, Some(lhs))?;
                self.release_all_temp_registers();
            }
            Instruction::AndRegReg(lhs, rhs) => {
                let prefix = InstructionPrefix::And;
                let lhs = self.get_register(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, lhs, rhs));
            }
            Instruction::AndLitReg(lhs, rhs) => {
                let prefix = InstructionPrefix::And;
                let lhs = self.get_register(lhs)?;

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, lhs, "!{var_name}"));
                    return Ok(());
                }

                self.generate_code(prefix, rhs, Some(lhs))?;
                self.release_all_temp_registers();
            }
            Instruction::OrRegReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Or;
                let lhs = self.get_register(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, lhs, rhs));
            }
            Instruction::OrLitReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Or;
                let lhs = self.get_register(lhs)?;

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, lhs, "!{var_name}"));
                    return Ok(());
                }

                self.generate_code(prefix, rhs, Some(lhs))?;
                self.release_all_temp_registers();
            }
            Instruction::XorRegReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Xor;
                let lhs = self.get_register(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, lhs, rhs));
            }
            Instruction::XorLitReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Xor;
                let lhs = self.get_register(lhs)?;

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, lhs, "!{var_name}"));
                    return Ok(());
                }

                self.generate_code(prefix, rhs, Some(lhs))?;
                self.release_all_temp_registers();
            }
            Instruction::Not(reg) => {
                let prefix = InstructionPrefix::Not;
                let reg = self.get_register(reg)?;
                self.code.push(formatted!(prefix, reg));
            }
            Instruction::PshReg(reg) => {
                let prefix = InstructionPrefix::Psh;
                let reg = self.get_register(reg)?;
                self.code.push(formatted!(prefix, reg));
            }
            Instruction::PshLit(lit) => {
                let prefix = InstructionPrefix::Psh;

                if let Statement::Var(offset) = lit {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, "!{var_name}"));
                    return Ok(());
                }

                if let Statement::HexLiteral(_) = lit {
                    let hex = self.gen_hex_lit(lit)?;
                    self.code.push(formatted!(prefix, hex));
                    return Ok(());
                };

                let result = self.generate_code(prefix, lit, None)?;
                self.code.push(formatted!(prefix, result));
                self.release_all_temp_registers();
            }
            Instruction::Pop(reg) => {
                let prefix = InstructionPrefix::Pop;
                let reg = self.get_register(reg)?;
                self.code.push(formatted!(prefix, reg));
            }
            Instruction::Call(address) => {
                let prefix = InstructionPrefix::Call;

                let Statement::Address(inner) = address else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        address.offset(),
                    );
                };

                if let Statement::BinaryOp { .. } = inner.as_ref() {
                    let rhs = self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?;
                    self.code.push(formatted!(prefix, "&[{rhs}]"));
                    self.release_all_temp_registers();
                    return Ok(());
                }

                let rhs = self.get_address(address)?;
                self.code.push(formatted!(prefix, "&[{rhs}]"));
            }
            Instruction::Ret(_) => {
                let prefix = InstructionPrefix::Ret;
                self.code.push(prefix.to_string());
            }
            Instruction::JeqReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Jeq;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                if let Statement::BinaryOp { .. } = inner.as_ref() {
                    let lhs = self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?;
                    let rhs = self.get_register(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
                    self.release_all_temp_registers();
                    return Ok(());
                }

                let lhs = self.get_address(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
            }
            Instruction::JeqLit(lhs, rhs) => {
                let prefix = InstructionPrefix::Jeq;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                let lhs = if let Statement::BinaryOp { .. } = inner.as_ref() {
                    self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?
                        .to_string()
                } else {
                    self.get_address(lhs)?
                };

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, "&[{lhs}]", "!{var_name}"));
                    return Ok(());
                }

                if let Statement::HexLiteral(_) = rhs {
                    let hex = self.gen_hex_lit(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", hex));
                    return Ok(());
                };

                let rhs = self.generate_code(prefix, rhs, None)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
                self.release_all_temp_registers();
            }
            Instruction::JgtReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Jgt;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                if let Statement::BinaryOp { .. } = inner.as_ref() {
                    let lhs = self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?;
                    let rhs = self.get_register(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
                    self.release_all_temp_registers();
                    return Ok(());
                }

                let lhs = self.get_address(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
            }
            Instruction::JgtLit(lhs, rhs) => {
                let prefix = InstructionPrefix::Jgt;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                let lhs = if let Statement::BinaryOp { .. } = inner.as_ref() {
                    self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?
                        .to_string()
                } else {
                    self.get_address(lhs)?
                };

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, "&[{lhs}]", "!{var_name}"));
                    return Ok(());
                }

                if let Statement::HexLiteral(_) = rhs {
                    let hex = self.gen_hex_lit(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", hex));
                    return Ok(());
                };

                let rhs = self.generate_code(prefix, rhs, None)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
                self.release_all_temp_registers();
            }
            Instruction::JneReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Jne;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                if let Statement::BinaryOp { .. } = inner.as_ref() {
                    let lhs = self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?;
                    let rhs = self.get_register(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
                    self.release_all_temp_registers();
                    return Ok(());
                }

                let lhs = self.get_address(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
            }
            Instruction::JneLit(lhs, rhs) => {
                let prefix = InstructionPrefix::Jne;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                let lhs = if let Statement::BinaryOp { .. } = inner.as_ref() {
                    self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?
                        .to_string()
                } else {
                    self.get_address(lhs)?
                };

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, "&[{lhs}]", "!{var_name}"));
                    return Ok(());
                }

                if let Statement::HexLiteral(_) = rhs {
                    let hex = self.gen_hex_lit(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", hex));
                    return Ok(());
                };

                let rhs = self.generate_code(prefix, rhs, None)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
                self.release_all_temp_registers();
            }
            Instruction::JgeReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Jge;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                if let Statement::BinaryOp { .. } = inner.as_ref() {
                    let lhs = self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?;
                    let rhs = self.get_register(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
                    self.release_all_temp_registers();
                    return Ok(());
                }

                let lhs = self.get_address(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
            }
            Instruction::JgeLit(lhs, rhs) => {
                let prefix = InstructionPrefix::Jge;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                let lhs = if let Statement::BinaryOp { .. } = inner.as_ref() {
                    self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?
                        .to_string()
                } else {
                    self.get_address(lhs)?
                };

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, "&[{lhs}]", "!{var_name}"));
                    return Ok(());
                }

                if let Statement::HexLiteral(_) = rhs {
                    let hex = self.gen_hex_lit(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", hex));
                    return Ok(());
                };

                let rhs = self.generate_code(prefix, rhs, None)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
                self.release_all_temp_registers();
            }
            Instruction::JleReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Jle;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                if let Statement::BinaryOp { .. } = inner.as_ref() {
                    let lhs = self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?;
                    let rhs = self.get_register(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
                    self.release_all_temp_registers();
                    return Ok(());
                }

                let lhs = self.get_address(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
            }
            Instruction::JltLit(lhs, rhs) => {
                let prefix = InstructionPrefix::Jlt;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                let lhs = if let Statement::BinaryOp { .. } = inner.as_ref() {
                    self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?
                        .to_string()
                } else {
                    self.get_address(lhs)?
                };

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, "&[{lhs}]", "!{var_name}"));
                    return Ok(());
                }

                if let Statement::HexLiteral(_) = rhs {
                    let hex = self.gen_hex_lit(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", hex));
                    return Ok(());
                };

                let rhs = self.generate_code(prefix, rhs, None)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
                self.release_all_temp_registers();
            }
            Instruction::JltReg(lhs, rhs) => {
                let prefix = InstructionPrefix::Jlt;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                if let Statement::BinaryOp { .. } = inner.as_ref() {
                    let lhs = self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?;
                    let rhs = self.get_register(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
                    self.release_all_temp_registers();
                    return Ok(());
                }

                let lhs = self.get_address(lhs)?;
                let rhs = self.get_register(rhs)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
            }
            Instruction::JleLit(lhs, rhs) => {
                let prefix = InstructionPrefix::Jle;

                let Statement::Address(inner) = lhs else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        lhs.offset(),
                    );
                };

                let lhs = if let Statement::BinaryOp { .. } = inner.as_ref() {
                    self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?
                        .to_string()
                } else {
                    self.get_address(lhs)?
                };

                if let Statement::Var(offset) = rhs {
                    let var_name = offset.get_source(&self.source);
                    self.code.push(formatted!(prefix, "&[{lhs}]", "!{var_name}"));
                    return Ok(());
                }

                if let Statement::HexLiteral(_) = rhs {
                    let hex = self.gen_hex_lit(rhs)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]", hex));
                    return Ok(());
                };

                let rhs = self.generate_code(prefix, rhs, None)?;
                self.code.push(formatted!(prefix, "&[{lhs}]", rhs));
                self.release_all_temp_registers();
            }
            Instruction::Jmp(address) => {
                let prefix = InstructionPrefix::Jmp;

                let Statement::Address(inner) = address else {
                    return unexpected_statement(
                        self.source,
                        "unexpected statement, expected: [ADDRESS]",
                        address.offset(),
                    );
                };

                if let Statement::BinaryOp { .. } = inner.as_ref() {
                    let lhs = self.generate_code(InstructionPrefix::Mov, inner.as_ref(), None)?;
                    self.code.push(formatted!(prefix, "&[{lhs}]"));
                    self.release_all_temp_registers();
                    return Ok(());
                };

                let address = self.get_address(address)?;
                self.code.push(formatted!(prefix, "&[{address}]"));
                self.release_all_temp_registers();
            }
            Instruction::Hlt(_) => {
                let prefix = InstructionPrefix::Hlt;
                self.code.push(prefix.to_string());
            }
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
        let mut codegen = CodeGenerator::new(&source, &ast).with_module(&module);
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
    fn test_gen_mov_reg_reg() {
        let source = "mov r1, r2";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "MOV R1, R2");
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
MOV R7, R2
ADD R8, R7
MOV R1, R8
POP R7
POP R8"#
        );
    }

    #[test]
    fn test_gen_mov_reg_mem() {
        let source = "mov &[$c0d3], r2";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "MOV &[$C0D3], R2");

        let source = "mov &[!var], r2";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "MOV &[!var], R2");

        let source = "mov &[$c0d3 + r2], r2";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
ADD R8, R7
MOV &[R8], R2
POP R7
POP R8"#
        );
    }

    #[test]
    fn test_gen_mov_mem_reg() {
        let source = "mov r2, &[$c0d3]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "MOV R2, &[$C0D3]");

        let source = "mov r2, &[!var]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "MOV R2, &[!var]");

        let source = "mov r2, &[$c0d3 + r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
ADD R8, R7
MOV R2, &[R8]
POP R7
POP R8"#
        );
    }

    #[test]
    fn test_gen_mov_lit_mem() {
        let source = "mov &[$c0d3], $c0d3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
POP R8
MOV &[$C0D3], R8"#
        );

        let source = "mov &[!var], $c0d3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
POP R8
MOV &[!var], R8"#
        );

        let source = "mov &[$c0d3 + r2], $c0d3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
ADD R8, R7
PSH R6
MOV R6, $C0D3
POP R6
POP R7
POP R8
MOV &[R8], R6"#
        );

        let source = "mov &[$c0d3], !var";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "MOV &[$C0D3], !var");

        let source = "mov &[$c0d3], [$c0d3 + r2 + !var]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
PSH R6
MOV R6, !var
ADD R7, R6
ADD R8, R7
POP R6
POP R7
POP R8
MOV &[$C0D3], R8"#
        );

        let source = "mov &[!var], [$c0d3 + r2 + !var]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
PSH R6
MOV R6, !var
ADD R7, R6
ADD R8, R7
POP R6
POP R7
POP R8
MOV &[!var], R8"#
        );

        let source = "mov &[r2], &[r3]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "MOV &[r2], &[r3]");
    }

    #[test]
    #[should_panic]
    fn test_gen_too_large() {
        let source = "mov &[!var + $c0d3 + r2], [$c0d3 + r2 + !var]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, !var
PSH R7
MOV R7, $C0D3
PSH R6
MOV R6, R2
ADD R7, R6
ADD R8, R7
PSH R5
MOV R5, $C0D3
PSH R4
MOV R4, R2
PSH R3
MOV R3, !var
ADD R4, R3
ADD R5, R4
POP R3
POP R4
POP R5
POP R6
POP R7
POP R8
MOV &[R8], R5"#
        );
    }

    #[test]
    fn test_add_reg_reg() {
        let source = "add r2, r3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "ADD R2, R3");
    }

    #[test]
    fn test_add_lit_reg() {
        let source = "add r2, $c0d3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "ADD R2, $C0D3");

        let source = "add r1, [$c0d3 + r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
ADD R8, R7
ADD R1, R8
POP R7
POP R8"#
        );

        let source = "add r1, !var";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "ADD R1, !var");
    }

    #[test]
    fn test_sub_reg_reg() {
        let source = "sub r2, r3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "SUB R2, R3");
    }

    #[test]
    fn test_sub_lit_reg() {
        let source = "sub r2, $c0d3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "SUB R2, $C0D3");

        let source = "sub r1, [$c0d3 + r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
ADD R8, R7
SUB R1, R8
POP R7
POP R8"#
        );

        let source = "sub r1, !var";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "SUB R1, !var");
    }

    #[test]
    fn test_mul_reg_reg() {
        let source = "mul r2, r3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "MUL R2, R3");
    }

    #[test]
    fn test_mul_lit_reg() {
        let source = "mul r2, $c0d3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "MUL R2, $C0D3");

        let source = "mul r1, [$c0d3 * r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
MUL R8, R7
MUL R1, R8
POP R7
POP R8"#
        );

        let source = "mul r1, !var";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "MUL R1, !var");
    }

    #[test]
    fn test_gen_inc() {
        let source = "inc r2";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "INC R2");
    }

    #[test]
    fn test_gen_dec() {
        let source = "dec r2";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "DEC R2");
    }

    #[test]
    fn test_lsh_reg_reg() {
        let source = "lsh r2, r3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "LSH R2, R3");
    }

    #[test]
    fn test_lsh_lit_reg() {
        let source = "lsh r2, $c0d3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "LSH R2, $C0D3");

        let source = "lsh r1, [$c0d3 - r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
SUB R8, R7
LSH R1, R8
POP R7
POP R8"#
        );

        let source = "lsh r1, !var";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "LSH R1, !var");
    }

    #[test]
    fn test_rsh_reg_reg() {
        let source = "rsh r2, r3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "RSH R2, R3");
    }

    #[test]
    fn test_rsh_lit_reg() {
        let source = "rsh r2, $c0d3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "RSH R2, $C0D3");

        let source = "rsh r1, [$c0d3 - r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
SUB R8, R7
RSH R1, R8
POP R7
POP R8"#
        );

        let source = "rsh r1, !var";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "RSH R1, !var");
    }

    #[test]
    fn test_and_reg_reg() {
        let source = "and r2, r3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "AND R2, R3");
    }

    #[test]
    fn test_and_lit_reg() {
        let source = "and r2, $c0d3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "AND R2, $C0D3");

        let source = "and r1, [$c0d3 - r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
SUB R8, R7
AND R1, R8
POP R7
POP R8"#
        );

        let source = "and r1, !var";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "AND R1, !var");
    }

    #[test]
    fn test_or_reg_reg() {
        let source = "or r2, r3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "OR R2, R3");
    }

    #[test]
    fn test_or_lit_reg() {
        let source = "or r2, $c0d3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "OR R2, $C0D3");

        let source = "or r1, [$c0d3 - r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
SUB R8, R7
OR R1, R8
POP R7
POP R8"#
        );

        let source = "or r1, !var";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "OR R1, !var");
    }

    #[test]
    fn test_xor_reg_reg() {
        let source = "xor r2, r3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "XOR R2, R3");
    }

    #[test]
    fn test_xor_lit_reg() {
        let source = "xor r2, $c0d3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "XOR R2, $C0D3");

        let source = "xor r1, [$c0d3 - r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
SUB R8, R7
XOR R1, R8
POP R7
POP R8"#
        );

        let source = "xor r1, !var";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "XOR R1, !var");
    }

    #[test]
    fn test_gen_not() {
        let source = "not r2";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "NOT R2");
    }

    #[test]
    fn test_push_reg() {
        let source = "psh r2";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "PSH R2");
    }

    #[test]
    fn test_psh_lit() {
        let source = "psh $c0d3";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "PSH $C0D3");

        let source = "psh [$c0d3 + r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
ADD R8, R7
PSH R8
POP R7
POP R8"#
        );

        let source = "psh !var";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "PSH !var");
    }

    #[test]
    fn test_gen_pop() {
        let source = "pop r2";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "POP R2");
    }

    #[test]
    fn test_gen_call() {
        let source = "call &[$c0d3]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "CALL &[$C0D3]");

        let source = "call &[$0303 + r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $0303
PSH R7
MOV R7, R2
ADD R8, R7
CALL &[R8]
POP R7
POP R8"#
        );

        let source = "call &[!var]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "CALL &[!var]");

        let source = "call &[!var + r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, !var
PSH R7
MOV R7, R2
ADD R8, R7
CALL &[R8]
POP R7
POP R8"#
        );
    }

    #[test]
    fn test_ret() {
        let source = "ret";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "RET");
    }

    #[test]
    fn test_jeq_reg() {
        let source = "jeq &[$c0d3], r2";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "JEQ &[$C0D3], R2");

        let source = "jeq &[!var], r2";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "JEQ &[!var], R2");

        let source = "jeq &[$c0d3 + r2], r2";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
ADD R8, R7
JEQ &[R8], R2
POP R7
POP R8"#
        );
    }

    #[test]
    fn test_jeq_lit() {
        let source = "jeq &[$c0d3], $0303";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "JEQ &[$C0D3], $0303");

        let source = "jeq &[$c0d3], !var";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "JEQ &[$C0D3], !var");

        let source = "jeq &[$c0d3], [$0303 + r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $0303
PSH R7
MOV R7, R2
ADD R8, R7
JEQ &[$C0D3], R8
POP R7
POP R8"#
        );

        let source = "jeq &[$c0d3 + r2], [$0303 + r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
ADD R8, R7
PSH R6
MOV R6, $0303
PSH R5
MOV R5, R2
ADD R6, R5
JEQ &[R8], R6
POP R5
POP R6
POP R7
POP R8"#
        );
    }

    #[test]
    fn test_gen_jmp() {
        let source = "jmp &[$c0d3]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "JMP &[$C0D3]");

        let source = "jmp &[$c0d3 + r2]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(
            result,
            r#"PSH R8
MOV R8, $C0D3
PSH R7
MOV R7, R2
ADD R8, R7
JMP &[R8]
POP R7
POP R8"#
        );

        let source = "jmp &[!var]";
        let ast = crate::parser::parse(source).unwrap();
        let mut generator = CodeGenerator::new(source, &ast);

        generator.generate().unwrap();
        let result = generator.to_string();
        assert_eq!(result, "JMP &[!var]");
    }
}
