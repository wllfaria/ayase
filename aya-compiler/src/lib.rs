mod file;

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use aya_core::register::Register;
use aya_parser::{Ast, InstructionKind, Statement};

fn find_module(haystack: &[Module<'_>], needle: &PathBuf) -> usize {
    haystack.iter().position(|m| &m.path == needle).unwrap()
}

fn resolve_variable(name: &str, modules: &mut [Module<'_>], module_idx: usize) -> u16 {
    let module = &modules[module_idx];
    if let Some(address) = module.symbols.get(name) {
        return *address;
    }

    for import in &module.imports {
        let idx = find_module(modules, import);
        let import = &modules[idx];
        if let Some(address) = import.exports.get(name) {
            return *address;
        };
    }

    panic!("variable {name} doesnt exist");
}

fn encode_literal_or_address(
    source: &str,
    node: &Statement,
    modules: &mut [Module<'_>],
    module_idx: usize,
) -> (u8, u8) {
    let hex = match node {
        Statement::Var(name) => {
            let name = &source[name.start..name.end];
            resolve_variable(name, modules, module_idx)
        }
        Statement::HexLiteral(value) | Statement::Address(value) => {
            let value = &source[value.start..value.end];
            u16::from_str_radix(value, 16).expect("value out of bounds for a u16")
        }
        _ => unreachable!(),
    };

    let upper = ((hex & 0xff00) >> 8) as u8;
    let lower = (hex & 0x00ff) as u8;
    (lower, upper)
}

fn encode_register(source: &str, value: &Statement) -> u8 {
    let Statement::Register(name) = value else {
        unreachable!();
    };
    let name = &source[name.start..name.end];
    let register = Register::try_from(name).unwrap();
    register.into()
}

fn collect_symbols<'comp>(address: &mut u16, source: &'comp str, ast: &'comp Ast, module: &mut Module<'comp>) {
    for node in ast.statements.iter() {
        match node {
            Statement::Label { name, exported } => {
                let name = &source[name.start..name.end];
                module.symbols.insert(name, *address);
                if *exported {
                    module.exports.insert(name, *address);
                }
            }
            Statement::Const { name, value, exported } => {
                let Statement::HexLiteral(value) = value.as_ref() else {
                    unreachable!();
                };
                let name = &source[name.start..name.end];
                let value = &source[value.start..value.end];
                let value = u16::from_str_radix(value, 16).expect("number is larger than 16bits");
                module.symbols.insert(name, value);
                if *exported {
                    module.exports.insert(name, value);
                }
            }
            Statement::Data {
                name,
                values,
                size,
                exported,
            } => {
                let name = &source[name.start..name.end];
                module.symbols.insert(name, *address);
                let byte_size = if *size == 8 { 1 } else { 2 };
                let total_size = values.len() * byte_size;
                *address += total_size as u16;
                if *exported {
                    module.exports.insert(name, *address);
                }
            }
            Statement::Instruction(instr) => *address += instr.kind().byte_size() as u16,
            _ => {}
        }
    }
}

fn encode_data_8(source: &str, values: &[Statement], bytecode: &mut Vec<u8>) {
    for value in values {
        let Statement::HexLiteral(value) = value else {
            unreachable!();
        };
        let value = &source[value.start..value.end];
        let value = u8::from_str_radix(value, 16).expect("u8 out of range");
        bytecode.push(value);
    }
}

fn encode_data_16(source: &str, values: &[Statement], bytecode: &mut Vec<u8>) {
    for value in values {
        let Statement::HexLiteral(value) = value else {
            unreachable!();
        };
        let value = &source[value.start..value.end];
        let value = u16::from_str_radix(value, 16).expect("u8 out of range");
        let upper = ((value & 0xff00) >> 8) as u8;
        let lower = (value & 0x00ff) as u8;
        bytecode.push(lower);
        bytecode.push(upper);
    }
}

#[derive(Debug)]
struct Module<'comp> {
    _name: String,
    path: PathBuf,
    exports: HashMap<&'comp str, u16>,
    symbols: HashMap<&'comp str, u16>,
    imports: Vec<PathBuf>,
}

fn process_module(
    name: &str,
    path: PathBuf,
    code: String,
    modules: &mut Vec<Module<'_>>,
    asts: &mut Vec<Ast>,
    visited: &mut HashSet<PathBuf>,
    sources: &mut HashMap<PathBuf, String>,
) {
    if visited.contains(&path) {
        return;
    }
    visited.insert(path.clone());

    let ast = aya_parser::parse(&code).unwrap();

    let mut module = Module {
        _name: name.to_string(),
        path: path.clone(),
        exports: Default::default(),
        symbols: Default::default(),
        imports: Default::default(),
    };

    for node in ast.statements.iter() {
        if let Statement::Import { name, path, .. } = node {
            let name = &code[name.start..name.end];
            let path = &code[path.start..path.end];
            let code = file::load_module_from_path(path).unwrap();
            process_module(name, path.into(), code, modules, asts, visited, sources);
            module.imports.push(path.into());
        }
    }

    asts.push(ast);
    sources.insert(path, code);
    modules.push(module)
}

fn topological_sort<'a>(modules: &'a [Module<'a>]) -> Vec<usize> {
    let mut sorted = Vec::with_capacity(modules.len());

    let mut idx_map = HashMap::with_capacity(modules.len());
    for (idx, module) in modules.iter().enumerate() {
        idx_map.insert(&module.path, idx);
    }

    let mut in_degrees = vec![0; modules.len()];
    for module in modules {
        for import in &module.imports {
            if let Some(&idx) = idx_map.get(import) {
                in_degrees[idx] += 1;
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
            if let Some(&idx) = idx_map.get(import) {
                in_degrees[idx] -= 1;
                if in_degrees[idx] == 0 {
                    queue.push_back(idx);
                }
            }
        }
    }

    sorted.reverse();
    sorted
}

fn compile_module<'comp>(
    source: &'comp str,
    module_ast: &'comp Ast,
    module_idx: usize,
    modules: &mut [Module<'comp>],
    bytecode: &mut Vec<u8>,
) {
    let module = &mut modules[module_idx];
    let mut address = bytecode.len() as u16;
    collect_symbols(&mut address, source, module_ast, module);

    for node in module_ast.statements.iter() {
        if let Statement::Data { size, values, .. } = node {
            if *size == 8 {
                encode_data_8(source, values, bytecode);
            } else {
                encode_data_16(source, values, bytecode);
            }
        };

        let Statement::Instruction(instruction) = node else {
            continue;
        };

        bytecode.push(instruction.opcode().into());

        match instruction.kind() {
            InstructionKind::LitReg | InstructionKind::MemReg => {
                let lhs = instruction.lhs();
                let rhs = instruction.rhs();
                let register = encode_register(source, lhs);
                let (lower, upper) = encode_literal_or_address(source, rhs, modules, module_idx);
                bytecode.push(register);
                bytecode.push(lower);
                bytecode.push(upper);
            }
            InstructionKind::RegLit | InstructionKind::RegMem => {
                let lhs = instruction.lhs();
                let rhs = instruction.rhs();
                let (lower, upper) = encode_literal_or_address(source, lhs, modules, module_idx);
                let register = encode_register(source, rhs);
                bytecode.push(lower);
                bytecode.push(upper);
                bytecode.push(register);
            }
            InstructionKind::RegReg | InstructionKind::RegPtrReg => {
                let lhs = instruction.lhs();
                let rhs = instruction.rhs();
                let dest = encode_register(source, lhs);
                let from = encode_register(source, rhs);
                bytecode.push(dest);
                bytecode.push(from);
            }
            InstructionKind::LitMem => {
                let lhs = instruction.lhs();
                let rhs = instruction.rhs();
                let (lower, upper) = encode_literal_or_address(source, lhs, modules, module_idx);
                bytecode.push(lower);
                bytecode.push(upper);
                let (lower, upper) = encode_literal_or_address(source, rhs, modules, module_idx);
                bytecode.push(lower);
                bytecode.push(upper);
            }
            InstructionKind::SingleReg => {
                let lhs = instruction.lhs();
                let register = encode_register(source, lhs);
                bytecode.push(register);
            }
            InstructionKind::SingleLit => {
                let lhs = instruction.lhs();
                let (lower, upper) = encode_literal_or_address(source, lhs, modules, module_idx);
                bytecode.push(lower);
                bytecode.push(upper);
            }
            InstructionKind::NoArgs => {}
        }
    }
}

fn compile_inner<P: AsRef<Path> + std::fmt::Debug>(code: String, path: P) -> (u16, Vec<u8>) {
    let mut sources = HashMap::new();
    let mut modules = vec![];
    let mut asts = vec![];
    let mut visited = HashSet::default();
    let mut bytecode = vec![];
    let path = path.as_ref().to_path_buf();

    process_module(
        "main",
        path.clone(),
        code,
        &mut modules,
        &mut asts,
        &mut visited,
        &mut sources,
    );

    let sorted = topological_sort(&modules);

    for idx in sorted {
        let module = &mut modules[idx];
        let source = sources.get(&module.path).unwrap();
        let ast = &asts[idx];
        compile_module(source, ast, idx, &mut modules, &mut bytecode);
    }

    let start = modules
        .iter()
        .find(|m| m.path == path)
        .unwrap()
        .symbols
        .get("start")
        .copied()
        .unwrap_or_default();

    (start, bytecode)
}

pub fn compile<P: AsRef<Path> + std::fmt::Debug>(path: P) -> (u16, Vec<u8>) {
    let code = file::load_module_from_path(path.as_ref()).unwrap();
    compile_inner(code, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        let program = r#"
            mov r1, $4200
            mov &[$0060], r1
            mov r1, $1300
            mov r2, &[$0060]
            add r1, r2
        "#;

        let result = compile_inner(program.into(), "test.aya");
        let result = format!("{result:#04X?}");
        insta::assert_snapshot!(result);
    }

    #[test]
    fn test_with_labels() {
        let program = r#"
            start:
                mov &[$0050], $0A
            loop:
                mov acc, &[$0050]
                dec acc
                mov &[$0050], acc
                inc r2
                inc r2
                inc r2
                jne &[!loop], $00
            end:
                hlt
        "#;

        let result = compile_inner(program.into(), "test.aya");
        let result = format!("{result:#04X?}");
        insta::assert_snapshot!(result);
    }

    #[test]
    fn constants_and_data_blocks() {
        // weird spacing here is intended.
        let program = r#"
            const some_const = $C0D3
            +const other_const = $FEFE

            +data8 bytes = { $01,   $02,$03  ,    $04   }
            data16 bytes = { $1234,   $5678,$9ABC  ,    $DEF0   }

            start:
                mov &[$1234], !some_const
                mov &[$5678], !other_const
        "#;
        let result = compile_inner(program.into(), "test.aya");
        let result = format!("{result:#04X?}");
        insta::assert_snapshot!(result);
    }
}
