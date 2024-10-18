mod file;

use std::collections::HashMap;
use std::path::Path;

use aya_core::register::Register;
use aya_parser::{Ast, InstructionKind};

fn encode_literal_or_address<'parser>(atom: &'parser Ast<'parser>, symbol_table: &HashMap<&str, u16>) -> (u8, u8) {
    let hex = match atom {
        Ast::Var(name) => {
            let address = symbol_table.get(name).expect("undefined label");
            *address
        }
        Ast::HexLiteral(value) | Ast::Address(value) => {
            u16::from_str_radix(value, 16).expect("value out of bounds for a u16")
        }
        _ => unreachable!(),
    };

    let upper = ((hex & 0xff00) >> 8) as u8;
    let lower = (hex & 0x00ff) as u8;
    (lower, upper)
}

fn encode_register<'parser>(value: &'parser Ast<'parser>) -> u8 {
    let Ast::Register(name) = value else {
        unreachable!();
    };
    let register = Register::try_from(*name).unwrap();
    register.into()
}

pub fn collect_symbols<'assembler>(ast: &'assembler [Ast<'assembler>]) -> HashMap<&'assembler str, u16> {
    let mut symbols: HashMap<&str, u16> = HashMap::default();
    let mut current_address = 0;

    for node in ast {
        match node {
            Ast::Label(name) => _ = symbols.insert(name, current_address),
            Ast::Const { name, value, .. } => {
                let Ast::HexLiteral(val) = value.as_ref() else {
                    unreachable!();
                };
                symbols.insert(
                    name,
                    u16::from_str_radix(val, 16).expect("number is larger than 16bits"),
                );
            }
            Ast::Data { name, values, size, .. } => {
                symbols.insert(name, current_address);
                let byte_size = if *size == 8 { 1 } else { 2 };
                let total_size = values.len() * byte_size;
                current_address += total_size as u16;
            }
            Ast::Instruction(instr) => current_address += instr.kind().byte_size() as u16,
            _ => {}
        }
    }

    symbols
}

fn encode_data_8(values: &[Ast], bytecode: &mut Vec<u8>) {
    for value in values {
        let Ast::HexLiteral(value) = value else {
            unreachable!();
        };
        let value = u8::from_str_radix(value, 16).expect("u8 out of range");
        bytecode.push(value);
    }
}
fn encode_data_16(values: &[Ast], bytecode: &mut Vec<u8>) {
    for value in values {
        let Ast::HexLiteral(value) = value else {
            unreachable!();
        };
        let value = u16::from_str_radix(value, 16).expect("u8 out of range");
        let upper = ((value & 0xff00) >> 8) as u8;
        let lower = (value & 0x00ff) as u8;
        bytecode.push(lower);
        bytecode.push(upper);
    }
}

// NOTE:
// when compiling the high level steps are:
// 1.   Parse the source file which this program was called with, that is the main module
//
// 2.   From the ast generated on the previous step, go over every import.
//
// 2.1  when parsing an import, we will generate an AST for that module aswell, and add
//      an entry of that module as a dependency of the module that imported it, also
//      storing this as an already parsed module, so that if other modules also import it
//      we avoid parsing it again.
//
// 2.2  from that, we should recursively parse every module thats on the program tree of
//      imports, which will make us a list of modules and their dependencies.
//
//      We will use that list to sort modules based on its dependencies, so we can start
//      module resolution from modules that have no dependencies and walk our way up to
//      the modules with more dependencies.
//
// 3.   when resolving a module, we need to keep track of which labels, constants or data
//      the given module exports, as items not exported cannot be imported by other
//      modules.
//
// 3.1  we will walk the resolution from the module with less dependencies to the module
//      with more dependencies, we should also never allow for cyclic dependencies.
//
// 3.2  cyclic dependencies are determined if a module depends on other module that also
//      depends on the former one, a visual representation would be something like:
//
//                          +-------+                +-------+
//                          | MOD A | -------------> | MOD B |
//                          +-------+                +-------+
//                              ^                        |
//                              |                        v
//                              |                    +-------+
//                              +------------------- | MOD C |
//                                                   +-------+

pub fn compile<S: AsRef<Path>>(source: S) -> Vec<u8> {
    let source = file::load_module_from_path(&source).unwrap();
    let ast = aya_parser::parse(&source);
    let mut main = Module { path: source.as_ref() };
    parse_module(&source);
    vec![]
    //let source = file::load_module_from_path(source).unwrap();
    //let ast = aya_parser::parse(&source);
    //let symbols = collect_symbols(&ast);
    //
    //let mut bytecode = vec![];
    //
    //for node in ast.iter() {
    //    if let Ast::Data { size, values, .. } = node {
    //        if *size == 8 {
    //            encode_data_8(values, &mut bytecode);
    //        } else {
    //            encode_data_16(values, &mut bytecode);
    //        }
    //    };
    //
    //    let Ast::Instruction(instruction) = node else {
    //        continue;
    //    };
    //
    //    bytecode.push(instruction.opcode().into());
    //
    //    match instruction.kind() {
    //        InstructionKind::LitReg | InstructionKind::MemReg => {
    //            let lhs = instruction.lhs();
    //            let rhs = instruction.rhs();
    //            let register = encode_register(lhs);
    //            let (lower, upper) = encode_literal_or_address(rhs, &symbols);
    //            bytecode.push(register);
    //            bytecode.push(lower);
    //            bytecode.push(upper);
    //        }
    //        InstructionKind::RegLit | InstructionKind::RegMem => {
    //            let lhs = instruction.lhs();
    //            let rhs = instruction.rhs();
    //            let (lower, upper) = encode_literal_or_address(lhs, &symbols);
    //            let register = encode_register(rhs);
    //            bytecode.push(lower);
    //            bytecode.push(upper);
    //            bytecode.push(register);
    //        }
    //        InstructionKind::RegReg | InstructionKind::RegPtrReg => {
    //            let lhs = instruction.lhs();
    //            let rhs = instruction.rhs();
    //            let dest = encode_register(lhs);
    //            let from = encode_register(rhs);
    //            bytecode.push(dest);
    //            bytecode.push(from);
    //        }
    //        InstructionKind::LitMem => {
    //            let lhs = instruction.lhs();
    //            let rhs = instruction.rhs();
    //            let (lower, upper) = encode_literal_or_address(lhs, &symbols);
    //            bytecode.push(lower);
    //            bytecode.push(upper);
    //            let (lower, upper) = encode_literal_or_address(rhs, &symbols);
    //            bytecode.push(lower);
    //            bytecode.push(upper);
    //        }
    //        InstructionKind::SingleReg => {
    //            let lhs = instruction.lhs();
    //            let register = encode_register(lhs);
    //            bytecode.push(register);
    //        }
    //        InstructionKind::SingleLit => {
    //            let lhs = instruction.lhs();
    //            let (lower, upper) = encode_literal_or_address(lhs, &symbols);
    //            bytecode.push(lower);
    //            bytecode.push(upper);
    //        }
    //        InstructionKind::NoArgs => {}
    //    }
    //}
    //
    //bytecode
}

#[derive(Debug)]
struct Module<'comp> {
    path: &'comp str,
}

fn parse_module<S>(source: S)
where
    S: AsRef<Path>,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        let program = r#"
            mov r1, $4200
            mov &0060, r1
            mov r1, $1300
            mov r2, &0060
            add r1, r2
        "#;

        let result = compile(program);
        let result = format!("{result:#04X?}");
        insta::assert_snapshot!(result);
    }

    #[test]
    fn test_with_labels() {
        let program = r#"
            start:
                mov &0050, $0A
            loop:
                mov ret, &0050
                dec ret
                mov &0050, ret
                inc r2
                inc r2
                inc r2
                jne &[!loop], $00
            end:
                hlt
        "#;

        let result = compile(program);
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
                mov &1234, [!some_const]
                mov &5678, [!other_const]
        "#;
        let result = compile(program);
        let result = format!("{result:#04X?}");
        insta::assert_snapshot!(result);
    }
}
