use aya_cpu::register::Register;

use crate::codegen::CodegenModule;
use crate::parser::ast::{Ast, ByteOffset, Instruction, InstructionKind, Statement};

fn encode_literal_or_address(module: &mut CodegenModule, node: &Statement, inst: &Instruction) -> miette::Result<u16> {
    match node {
        Statement::Var(name) => {
            let name_str = &module.code[name.start..name.end];

            if let Some(value) = module.symbols.get(name_str) {
                return Ok(*value);
            }

            if let Some(variables) = &module.variables {
                if let Some(value) = variables.get(name_str).as_ref() {
                    return Ok(value.to_value());
                }
            }

            let labels = vec![
                miette::LabeledSpan::at(*name, "this value"),
                miette::LabeledSpan::at(inst.offset(), "this statement"),
            ];
            Err(bail_multi(
                &module.code,
                labels,
                "[UNDEFINED_VARIABLE]: error while compiling statement",
                "variable is not defined or imported",
            ))
        }
        Statement::HexLiteral(value) => {
            let value_str = &module.code[value.start..value.end];
            let Ok(value) = u16::from_str_radix(value_str, 16) else {
                let labels = vec![
                    miette::LabeledSpan::at(*value, "this value"),
                    miette::LabeledSpan::at(inst.offset(), "this statement"),
                ];
                return Err(bail_multi(
                    &module.code,
                    labels,
                    "[INVALID_STATEMENT]: error while compiling statement",
                    "hex number is not within the u16 range",
                ));
            };

            Ok(value)
        }
        Statement::Address(value) => encode_literal_or_address(module, value.as_ref(), inst),
        _ => unreachable!(),
    }
}

fn encode_register(source: &str, value: &Statement) -> miette::Result<u8> {
    let Statement::Register(name) = value else {
        unreachable!();
    };
    let name_str = &source[name.start..name.end];
    match Register::try_from(name_str) {
        Ok(register) => Ok(register.into()),
        Err(_) => {
            let labels = vec![
                miette::LabeledSpan::at(*name, "this identifier"),
                miette::LabeledSpan::at(value.offset(), "this statement"),
            ];
            return Err(bail_multi(
                source,
                labels,
                "[INVALID_STATEMENT]: error while compiling statement",
                "hex number is not within the u8 range",
            ));
        }
    }
}

fn collect_symbols(module: &mut CodegenModule, ast: &Ast, address: &mut u16) {
    for node in ast.statements.iter() {
        match node {
            Statement::Label { name, exported } => {
                let name = &module.code[name.start..name.end];
                module.symbols.insert(name.into(), *address);
                if *exported {
                    module.exports.insert(name.into(), *address);
                }
            }
            Statement::Data {
                name,
                values,
                size,
                exported,
            } => {
                let name = &module.code[name.start..name.end];
                module.symbols.insert(name.into(), *address);
                let byte_size = if *size == 8 { 1 } else { 2 };
                let total_size = values.len() * byte_size;
                *address += total_size as u16;
                if *exported {
                    module.exports.insert(name.into(), *address);
                }
            }
            Statement::Instruction(instr) => *address += instr.kind().byte_size() as u16,
            _ => {}
        }
    }
}

fn bail_multi<S: AsRef<str>>(
    source: &str,
    labels: impl IntoIterator<Item = miette::LabeledSpan>,
    message: S,
    help: S,
) -> miette::Error {
    miette::Error::from(
        miette::MietteDiagnostic::new(message.as_ref())
            .with_labels(labels)
            .with_help(help.as_ref()),
    )
    .with_source_code(source.to_string())
}

fn compile_data_block(module: &mut CodegenModule, stat: &Statement, bytecode: &mut Vec<u8>) -> miette::Result<()> {
    let Statement::Data { size, values, .. } = stat else {
        unreachable!();
    };

    match size {
        8 => {
            for value in values {
                let Statement::HexLiteral(value) = value else {
                    unreachable!();
                };
                let value_str = &module.code[value.start..value.end];
                let Ok(value_hex) = u8::from_str_radix(value_str, 16) else {
                    let labels = vec![
                        miette::LabeledSpan::at(*value, "this value"),
                        miette::LabeledSpan::at(stat.offset(), "this statement"),
                    ];
                    return Err(bail_multi(
                        &module.code,
                        labels,
                        "[INVALID_STATEMENT]: error while compiling statement",
                        "hex number is not within the u8 range",
                    ));
                };
                bytecode.push(value_hex);
            }
        }
        16 => {
            for value in values {
                let Statement::HexLiteral(value) = value else {
                    unreachable!();
                };
                let value_str = &module.code[value.start..value.end];
                let Ok(value_hex) = u16::from_str_radix(value_str, 16) else {
                    let labels = vec![
                        miette::LabeledSpan::at(*value, "this value"),
                        miette::LabeledSpan::at(stat.offset(), "this statement"),
                    ];
                    return Err(bail_multi(
                        &module.code,
                        labels,
                        "[INVALID_STATEMENT]: error while compiling statement",
                        "hex number is not within the u16 range",
                    ));
                };
                let [lower, upper] = value_hex.to_le_bytes();
                bytecode.push(lower);
                bytecode.push(upper);
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn compile_instruction(module: &mut CodegenModule, inst: &Instruction, bytecode: &mut Vec<u8>) -> miette::Result<()> {
    bytecode.push(inst.opcode().into());

    match inst.kind() {
        InstructionKind::LitReg | InstructionKind::MemReg => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let register = encode_register(&module.code, lhs)?;
            let value = encode_literal_or_address(module, rhs, inst)?;
            let [lower, upper] = u16::to_le_bytes(value);
            bytecode.push(register);
            bytecode.push(lower);
            bytecode.push(upper);
        }
        InstructionKind::RegMem => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let value = encode_literal_or_address(module, lhs, inst)?;
            let [lower, upper] = u16::to_le_bytes(value);
            let register = encode_register(&module.code, rhs)?;
            bytecode.push(lower);
            bytecode.push(upper);
            bytecode.push(register);
        }
        InstructionKind::RegReg | InstructionKind::RegPtrReg => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let dest = encode_register(&module.code, lhs)?;
            let from = encode_register(&module.code, rhs)?;
            bytecode.push(dest);
            bytecode.push(from);
        }
        InstructionKind::LitMem => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let value = encode_literal_or_address(module, lhs, inst)?;
            let [lower, upper] = u16::to_le_bytes(value);
            bytecode.push(lower);
            bytecode.push(upper);
            let value = encode_literal_or_address(module, rhs, inst)?;
            let [lower, upper] = u16::to_le_bytes(value);
            bytecode.push(lower);
            bytecode.push(upper);
        }
        InstructionKind::SingleReg => {
            let lhs = inst.lhs();
            let register = encode_register(&module.code, lhs)?;
            bytecode.push(register);
        }
        InstructionKind::SingleLit => {
            let lhs = inst.lhs();
            let value = encode_literal_or_address(module, lhs, inst)?;
            let [lower, upper] = u16::to_le_bytes(value);
            bytecode.push(lower);
            bytecode.push(upper);
        }
        InstructionKind::NoArgs => {}
    };

    Ok(())
}

fn compile_module(module: &mut CodegenModule, ast: &Ast, bytecode: &mut Vec<u8>) -> miette::Result<()> {
    for node in ast.statements.iter() {
        match node {
            data @ Statement::Data { .. } => compile_data_block(module, data, bytecode)?,
            Statement::Instruction(inst) => compile_instruction(module, inst.as_ref(), bytecode)?,
            _ => {}
        }
    }
    Ok(())
}

pub fn compile(modules: Vec<CodegenModule>) -> miette::Result<Vec<u8>> {
    let mut bytecode = vec![];

    for mut module in modules {
        let mut address = bytecode.len() as u16;
        let ast = crate::parser::parse(&module.code)?;
        collect_symbols(&mut module, &ast, &mut address);
        compile_module(&mut module, &ast, &mut bytecode)?;
    }

    Ok(bytecode)
}

#[cfg(test)]
mod tests {
    use crate::AssembleBehavior;

    #[test]
    fn test_topological() {
        let result = crate::assemble("../samples/main.aya", AssembleBehavior::Bytecode).unwrap();
        println!("{result:?}");
        panic!();
    }
}
