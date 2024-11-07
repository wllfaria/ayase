use aya_cpu::register::Register;

use crate::codegen::CodegenModule;
use crate::parser::ast::{Ast, Instruction, InstructionKind, Statement};
use crate::utils::bail_multi;

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
        _ => unreachable!("{:?}", inst),
    }
}

fn encode_literal_byte(module: &mut CodegenModule, node: &Statement, inst: &Instruction) -> miette::Result<u8> {
    match node {
        Statement::Var(name) => {
            let name_str = &module.code[name.start..name.end];

            if let Some(value) = module.symbols.get(name_str) {
                if *value > 0xFF {
                    let labels = vec![
                        miette::LabeledSpan::at(*name, "this value"),
                        miette::LabeledSpan::at(inst.offset(), "this statement"),
                    ];
                    return Err(bail_multi(
                        &module.code,
                        labels,
                        "[INVALID_STATEMENT]: error while compiling statement",
                        "hex number is not within u8 range",
                    ));
                }
                return Ok(*value as u8);
            }

            if let Some(variables) = &module.variables {
                if let Some(value) = variables.get(name_str).as_ref() {
                    return Ok(value.to_value_small());
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
            let Ok(value) = u8::from_str_radix(value_str, 16) else {
                let labels = vec![
                    miette::LabeledSpan::at(*value, "this value"),
                    miette::LabeledSpan::at(inst.offset(), "this statement"),
                ];
                return Err(bail_multi(
                    &module.code,
                    labels,
                    "[INVALID_STATEMENT]: error while compiling statement",
                    "hex number is not within the u8 range",
                ));
            };

            Ok(value)
        }
        _ => unreachable!("{:?}", inst),
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
            Err(bail_multi(
                source,
                labels,
                "[INVALID_STATEMENT]: error while compiling statement",
                "hex number is not within the u8 range",
            ))
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

fn compile_data_block(
    module: &mut CodegenModule,
    stat: &Statement,
    bytecode: &mut [u8; u16::MAX as usize],
    address: &mut u16,
) -> miette::Result<()> {
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
                bytecode[*address as usize] = value_hex;
                *address += 1;
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
                bytecode[*address as usize] = lower;
                *address += 1;
                bytecode[*address as usize] = upper;
                *address += 1;
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn compile_instruction(
    module: &mut CodegenModule,
    inst: &Instruction,
    bytecode: &mut [u8; u16::MAX as usize],
    address: &mut u16,
) -> miette::Result<()> {
    bytecode[*address as usize] = inst.opcode().into();
    *address += 1;

    match inst.kind() {
        InstructionKind::LitReg | InstructionKind::MemReg | InstructionKind::MemReg8 => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let register = encode_register(&module.code, lhs)?;
            let value = encode_literal_or_address(module, rhs, inst)?;
            let [lower, upper] = u16::to_le_bytes(value);
            bytecode[*address as usize] = register;
            *address += 1;
            bytecode[*address as usize] = lower;
            *address += 1;
            bytecode[*address as usize] = upper;
            *address += 1;
        }
        InstructionKind::LitReg8 => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let register = encode_register(&module.code, lhs)?;
            let value = encode_literal_byte(module, rhs, inst)?;
            bytecode[*address as usize] = register;
            *address += 1;
            bytecode[*address as usize] = value;
            *address += 1;
        }
        InstructionKind::LitMem8 => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let value = encode_literal_or_address(module, lhs, inst)?;
            let [lower, upper] = u16::to_le_bytes(value);
            bytecode[*address as usize] = lower;
            *address += 1;
            bytecode[*address as usize] = upper;
            *address += 1;
            let value = encode_literal_byte(module, rhs, inst)?;
            bytecode[*address as usize] = value;
            *address += 1;
        }
        InstructionKind::RegMem8 => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let Statement::Address(inner) = lhs else {
                unreachable!();
            };

            if let Statement::Register(_) = inner.as_ref() {
                let value = encode_register(&module.code, inner.as_ref())?;
                let register = encode_register(&module.code, rhs)?;
                bytecode[*address as usize] = value;
                *address += 1;
                bytecode[*address as usize] = register;
                *address += 1;
            } else {
                let value = encode_literal_or_address(module, lhs, inst)?;
                let [lower, upper] = u16::to_le_bytes(value);
                let register = encode_register(&module.code, rhs)?;
                bytecode[*address as usize] = lower;
                *address += 1;
                bytecode[*address as usize] = upper;
                *address += 1;
                bytecode[*address as usize] = register;
                *address += 1;
            }
        }
        InstructionKind::RegMem => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let Statement::Address(inner) = lhs else {
                unreachable!();
            };

            if let Statement::Register(_) = inner.as_ref() {
                let value = encode_register(&module.code, inner.as_ref())?;
                let register = encode_register(&module.code, rhs)?;
                bytecode[*address as usize] = value;
                *address += 1;
                bytecode[*address as usize] = 0;
                *address += 1;
                bytecode[*address as usize] = register;
                *address += 1;
            } else {
                let value = encode_literal_or_address(module, lhs, inst)?;
                let [lower, upper] = u16::to_le_bytes(value);
                let register = encode_register(&module.code, rhs)?;
                bytecode[*address as usize] = lower;
                *address += 1;
                bytecode[*address as usize] = upper;
                *address += 1;
                bytecode[*address as usize] = register;
                *address += 1;
            }
        }
        InstructionKind::RegReg | InstructionKind::RegPtrReg | InstructionKind::RegReg8 => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let dest = encode_register(&module.code, lhs)?;
            let from = encode_register(&module.code, rhs)?;
            bytecode[*address as usize] = dest;
            *address += 1;
            bytecode[*address as usize] = from;
            *address += 1;
        }
        InstructionKind::LitRegPtr => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();

            let Statement::Address(inner) = lhs else {
                unreachable!();
            };

            let reg = encode_register(&module.code, inner.as_ref())?;
            let lit = encode_literal_or_address(module, rhs, inst)?;
            let [lower, upper] = u16::to_le_bytes(lit);

            bytecode[*address as usize] = reg;
            *address += 1;
            bytecode[*address as usize] = lower;
            *address += 1;
            bytecode[*address as usize] = upper;
            *address += 1;
        }
        InstructionKind::LitMem => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let value = encode_literal_or_address(module, lhs, inst)?;
            let [lower, upper] = u16::to_le_bytes(value);
            bytecode[*address as usize] = lower;
            *address += 1;
            bytecode[*address as usize] = upper;
            *address += 1;
            let value = encode_literal_or_address(module, rhs, inst)?;
            let [lower, upper] = u16::to_le_bytes(value);
            bytecode[*address as usize] = lower;
            *address += 1;
            bytecode[*address as usize] = upper;
            *address += 1;
        }
        InstructionKind::SingleReg => {
            let lhs = inst.lhs();
            let register = encode_register(&module.code, lhs)?;
            bytecode[*address as usize] = register;
            *address += 1;
        }
        InstructionKind::SingleLit => {
            let lhs = inst.lhs();
            let value = encode_literal_or_address(module, lhs, inst)?;
            let [lower, upper] = u16::to_le_bytes(value);
            bytecode[*address as usize] = lower;
            *address += 1;
            bytecode[*address as usize] = upper;
            *address += 1;
        }
        InstructionKind::NoArgs => {}
    };

    Ok(())
}

fn compile_module(module: &mut CodegenModule, ast: &Ast, bytecode: &mut [u8; u16::MAX as usize]) -> miette::Result<()> {
    let mut start_address = module.address;
    for node in ast.statements.iter() {
        match node {
            data @ Statement::Data { .. } => compile_data_block(module, data, bytecode, &mut start_address)?,
            Statement::Instruction(inst) => compile_instruction(module, inst.as_ref(), bytecode, &mut start_address)?,
            _ => {}
        }
    }
    Ok(())
}

pub fn compile(mut modules: Vec<CodegenModule>) -> miette::Result<Vec<u8>> {
    let mut bytecode = [0; u16::MAX as usize];

    for module in modules.iter_mut() {
        let ast = crate::parser::parse(&module.code)?;
        let mut module_address = module.address;
        collect_symbols(module, &ast, &mut module_address);
        compile_module(module, &ast, &mut bytecode)?;
    }

    let last_address = bytecode.iter().rev().position(|&b| b != 0).unwrap_or(0);
    let last_address = u16::MAX as usize - last_address;
    let bytecode = bytecode[..last_address].to_vec();

    Ok(bytecode)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_compile() {
        let modules = vec![
            CodegenModule {
                name: "main".into(),
                path: "main.aya".into(),
                address: 0x0000,
                imports: vec![],
                symbols: HashMap::new(),
                variables: None,
                exports: HashMap::new(),
                code: [
                    "before_interrupt:",
                    "mov r1, $01",
                    "mov r2, $02",
                    "psh $0003",
                    "call_interrupt:",
                    "int $03",
                    "mask_interrupt_4:",
                    "mov r3, $01",
                    "lsh r3, $03",
                    "not r3",
                    "and im, acc",
                    "mov im, acc",
                    "call_interrupt_again:",
                    "int $03",
                    "this_should_run_instead:",
                    "mov r5, $05",
                ]
                .join("\n"),
            },
            CodegenModule {
                name: "other".into(),
                path: "other.aya".into(),
                address: 0x0064,
                imports: vec![],
                symbols: HashMap::new(),
                variables: None,
                exports: HashMap::new(),
                code: [
                    "data8 name = { $1 }",
                    "data8 lol = { $02 }",
                    "data16 lol2 = { $ffff }",
                    "interrupt_code:",
                    "mov r1, $FFFF",
                    "mov r2, $FFFF",
                    "xor r1, r2",
                    "lsh r1, r2",
                    "rti",
                ]
                .join("\n"),
            },
        ];

        let result = compile(modules).unwrap();

        assert_eq!(
            result,
            [
                0x11, 0x02, 0x01, 0x00, 0x11, 0x03, 0x02, 0x00, 0x41, 0x03, 0x00, 0xFD, 0x03, 0x00, 0x11, 0x04, 0x01,
                0x00, 0x31, 0x04, 0x03, 0x00, 0x3A, 0x04, 0x34, 0x0C, 0x00, 0x10, 0x0C, 0x00, 0xFD, 0x03, 0x00, 0x11,
                0x06, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02,
                0xFF, 0xFF, 0x11, 0x02, 0xFF, 0xFF, 0x11, 0x03, 0xFF, 0xFF, 0x38, 0x02, 0x03, 0x30, 0x02, 0x03, 0xFE,
            ]
        );
    }
}
