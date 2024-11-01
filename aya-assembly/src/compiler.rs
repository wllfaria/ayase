use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use aya_cpu::register::Register;

use crate::codegen::CodegenModule;
use crate::mod_resolver::Either;
use crate::parser::ast::{Ast, Instruction, InstructionKind, Statement};

//fn find_module(haystack: &[Module<'_>], needle: &PathBuf) -> usize {
//    haystack.iter().position(|m| &m.path == needle).unwrap()
//}

//fn resolve_variable(name: &str, modules: &mut [Module<'_>], module_idx: usize) -> u16 {
//    let module = &modules[module_idx];
//    if let Some(address) = module.symbols.get(name) {
//        return *address;
//    }
//
//    for import in &module.imports {
//        let idx = find_module(modules, import);
//        let import = &modules[idx];
//        if let Some(address) = import.exports.get(name) {
//            return *address;
//        };
//    }
//
//    panic!("variable {name} doesnt exist");
//}

fn encode_literal_or_address(module: &mut CodegenModule, node: &Statement) -> u16 {
    match node {
        Statement::Var(name) => {
            let name = &module.code[name.start..name.end];
            module.symbols.get(name).copied().unwrap()
        }
        Statement::HexLiteral(value) => {
            let value = &module.code[value.start..value.end];
            u16::from_str_radix(value, 16).expect("value out of bounds for a u16")
        }
        Statement::Address(value) => encode_literal_or_address(module, value.as_ref()),
        _ => unreachable!(),
    }
}

fn encode_register(source: &str, value: &Statement) -> u8 {
    let Statement::Register(name) = value else {
        unreachable!();
    };
    let name = &source[name.start..name.end];
    let register = Register::try_from(name).unwrap();
    register.into()
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

fn encode_data_8(source: &str, values: &[Statement], bytecode: &mut Vec<u8>) {}

fn encode_data_16(source: &str, values: &[Statement], bytecode: &mut Vec<u8>) {}

fn topological_sort(modules: &[CodegenModule]) -> Vec<usize> {
    let mut sorted = Vec::with_capacity(modules.len());

    let mut idx_path = HashMap::with_capacity(modules.len());
    for (idx, module) in modules.iter().enumerate() {
        idx_path.insert(&module.path, idx);
    }

    let mut idx_name = HashMap::with_capacity(modules.len());
    for (idx, module) in modules.iter().enumerate() {
        idx_name.insert(&module.name, idx);
    }

    let mut in_degrees = vec![0; modules.len()];
    for module in modules {
        for import in &module.imports {
            if let Some(&idx) = idx_path.get(import) {
                in_degrees[idx] += 1;
            }
        }

        if let Some(ref variables) = module.variables {
            for value in variables.values() {
                if let Either::ModuleField { module, .. } = value {
                    if let Some(&idx) = idx_name.get(&module) {
                        in_degrees[idx] += 1;
                    }
                }
            }
        }
    }

    let mut queue = VecDeque::new();
    for (index, degree) in in_degrees.iter().enumerate() {
        if *degree == 0 {
            queue.push_back(index);
        }
    }

    while let Some(idx) = queue.pop_front() {
        let module = &modules[idx];
        sorted.push(idx);

        for import in &module.imports {
            if let Some(&idx) = idx_path.get(import) {
                in_degrees[idx] -= 1;
                if in_degrees[idx] == 0 {
                    queue.push_back(idx);
                }
            }
        }

        if let Some(ref variables) = module.variables {
            for value in variables.values() {
                if let Either::ModuleField { module, .. } = value {
                    if let Some(&idx) = idx_name.get(&module) {
                        in_degrees[idx] -= 1;
                        if in_degrees[idx] == 0 {
                            queue.push_back(idx);
                        }
                    }
                }
            }
        }
    }

    if sorted.len() != modules.len() {
        panic!("cyclic dependency, probably");
    }

    sorted.reverse();
    sorted
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
            let register = encode_register(&module.code, lhs);
            let value = encode_literal_or_address(module, rhs);
            let [lower, upper] = u16::to_le_bytes(value);
            bytecode.push(register);
            bytecode.push(lower);
            bytecode.push(upper);
        }
        InstructionKind::RegMem => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let value = encode_literal_or_address(module, lhs);
            let [lower, upper] = u16::to_le_bytes(value);
            let register = encode_register(&module.code, rhs);
            bytecode.push(lower);
            bytecode.push(upper);
            bytecode.push(register);
        }
        InstructionKind::RegReg | InstructionKind::RegPtrReg => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let dest = encode_register(&module.code, lhs);
            let from = encode_register(&module.code, rhs);
            bytecode.push(dest);
            bytecode.push(from);
        }
        InstructionKind::LitMem => {
            let lhs = inst.lhs();
            let rhs = inst.rhs();
            let value = encode_literal_or_address(module, lhs);
            let [lower, upper] = u16::to_le_bytes(value);
            bytecode.push(lower);
            bytecode.push(upper);
            let value = encode_literal_or_address(module, rhs);
            let [lower, upper] = u16::to_le_bytes(value);
            bytecode.push(lower);
            bytecode.push(upper);
        }
        InstructionKind::SingleReg => {
            let lhs = inst.lhs();
            let register = encode_register(&module.code, lhs);
            bytecode.push(register);
        }
        InstructionKind::SingleLit => {
            let lhs = inst.lhs();
            let value = encode_literal_or_address(module, lhs);
            let [lower, upper] = u16::to_le_bytes(value);
            bytecode.push(lower);
            bytecode.push(upper);
        }
        InstructionKind::NoArgs => {}
    }
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

pub fn compile(mut modules: Vec<CodegenModule>) -> miette::Result<Vec<u8>> {
    let mut bytecode = vec![];
    let sorted = topological_sort(&modules);

    for idx in sorted {
        let module = &mut modules[idx];
        let mut address = bytecode.len() as u16;
        let ast = crate::parser::parse(&module.code)?;
        collect_symbols(module, &ast, &mut address);
        compile_module(module, &ast, &mut bytecode)?;
    }

    println!("{bytecode:?}");
    Ok(bytecode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AssembleBehavior;

    //#[test]
    //fn test_simple_program() {
    //    let program = r#"
    //        mov r1, $4200
    //        mov &[$0060], r1
    //        mov r1, $1300
    //        mov r2, &[$0060]
    //        add r1, r2
    //    "#;
    //
    //    let result = compile_inner(program.into(), "test.aya");
    //    let result = format!("{result:#04X?}");
    //    insta::assert_snapshot!(result);
    //}
    //
    //#[test]
    //fn test_with_labels() {
    //    let program = r#"
    //        start:
    //            mov &[$0050], $0A
    //        loop:
    //            mov acc, &[$0050]
    //            dec acc
    //            mov &[$0050], acc
    //            inc r2
    //            inc r2
    //            inc r2
    //            jne &[!loop], $00
    //        end:
    //            hlt
    //    "#;
    //
    //    let result = compile_inner(program.into(), "test.aya");
    //    let result = format!("{result:#04X?}");
    //    insta::assert_snapshot!(result);
    //}
    //
    //#[test]
    //fn constants_and_data_blocks() {
    //    // weird spacing here is intended.
    //    let program = r#"
    //        const some_const = $C0D3
    //        +const other_const = $FEFE
    //
    //        +data8 bytes = { $01,   $02,$03  ,    $04   }
    //        data16 bytes = { $1234,   $5678,$9ABC  ,    $DEF0   }
    //
    //        start:
    //            mov &[$1234], !some_const
    //            mov &[$5678], !other_const
    //    "#;
    //    let result = compile_inner(program.into(), "test.aya");
    //    let result = format!("{result:#04X?}");
    //    insta::assert_snapshot!(result);
    //}
    //
    //#[test]
    //#[should_panic]
    //fn invalid_data_range_u8() {
    //    let program = "data8 name = { $FF, $FFF }";
    //    compile_inner(program.into(), "test.aya").unwrap();
    //}
    //
    //#[test]
    //#[should_panic]
    //fn invalid_data_range_u16() {
    //    let program = "data16 name = { $FFFF, $FFFFF }";
    //    compile_inner(program.into(), "test.aya").unwrap();
    //}

    #[test]
    fn test_topological() {
        crate::assemble("../samples/main.aya", AssembleBehavior::Bytecode).unwrap();
        panic!();
    }
}
