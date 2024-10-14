use std::collections::HashMap;

use aya_core::register::Register;
use aya_parser::{Atom, Instruction, InstructionKind};

fn encode_literal_or_address<'parser>(atom: &'parser Atom<'parser>, symbol_table: &HashMap<&str, u16>) -> (u8, u8) {
    let hex = match atom {
        Atom::Var(name) => {
            let address = symbol_table.get(name).expect("undefined label");
            *address
        }
        Atom::HexLiteral(value) | Atom::Address(value) => {
            u16::from_str_radix(value, 16).expect("value out of bounds for a u16")
        }
        _ => unreachable!(),
    };

    let upper = ((hex & 0xff00) >> 8) as u8;
    let lower = (hex & 0x00ff) as u8;
    (lower, upper)
}

fn encode_register<'parser>(value: &'parser Atom<'parser>) -> u8 {
    let Atom::Register(name) = value else {
        unreachable!();
    };
    let register = Register::try_from(*name).unwrap();
    register.into()
}

pub fn collect_symbols<'assembler>(ast: &'assembler [Instruction<'assembler>]) -> HashMap<&'assembler str, u16> {
    let mut symbols: HashMap<&str, u16> = HashMap::default();
    let mut current_address = 0;

    for node in ast {
        match node {
            Instruction::Nop(Atom::Label(name)) => _ = symbols.insert(name, current_address),
            Instruction::Nop(Atom::Const { name, value, .. }) => {
                let Atom::HexLiteral(val) = value.as_ref() else {
                    unreachable!();
                };
                symbols.insert(
                    name,
                    u16::from_str_radix(val, 16).expect("number is larger than 16bits"),
                );
            }
            Instruction::Nop(Atom::Data { name, values, size, .. }) => {
                symbols.insert(name, current_address);
                let byte_size = if *size == 8 { 1 } else { 2 };
                let total_size = values.len() * byte_size;
                current_address += total_size as u16;
            }
            _ => current_address += node.kind().byte_size() as u16,
        }
    }

    symbols
}

fn encode_data_8(values: &[Atom], bytecode: &mut Vec<u8>) {
    for value in values {
        let Atom::HexLiteral(value) = value else {
            unreachable!();
        };
        let value = u8::from_str_radix(value, 16).expect("u8 out of range");
        bytecode.push(value);
    }
}
fn encode_data_16(values: &[Atom], bytecode: &mut Vec<u8>) {
    for value in values {
        let Atom::HexLiteral(value) = value else {
            unreachable!();
        };
        let value = u16::from_str_radix(value, 16).expect("u8 out of range");
        let upper = ((value & 0xff00) >> 8) as u8;
        let lower = (value & 0x00ff) as u8;
        bytecode.push(lower);
        bytecode.push(upper);
    }
}

pub fn compile(source: &str) -> Vec<u8> {
    let ast = aya_parser::parse(source);

    let labels = collect_symbols(&ast);

    let mut bytecode = vec![];

    for instruction in ast.iter() {
        if let Instruction::Nop(Atom::Data { size, values, .. }) = instruction {
            if *size == 8 {
                encode_data_8(values, &mut bytecode);
            } else {
                encode_data_16(values, &mut bytecode);
            }
        };

        if matches!(instruction.kind(), InstructionKind::Nop) {
            continue;
        }

        bytecode.push(instruction.opcode().into());

        match instruction.kind() {
            InstructionKind::LitReg | InstructionKind::MemReg => {
                let lhs = instruction.lhs();
                let rhs = instruction.rhs();
                let register = encode_register(lhs);
                let (lower, upper) = encode_literal_or_address(rhs, &labels);
                bytecode.push(register);
                bytecode.push(lower);
                bytecode.push(upper);
            }
            InstructionKind::RegLit | InstructionKind::RegMem => {
                let lhs = instruction.lhs();
                let rhs = instruction.rhs();
                let (lower, upper) = encode_literal_or_address(lhs, &labels);
                let register = encode_register(rhs);
                bytecode.push(lower);
                bytecode.push(upper);
                bytecode.push(register);
            }
            InstructionKind::RegReg | InstructionKind::RegPtrReg => {
                let lhs = instruction.lhs();
                let rhs = instruction.rhs();
                let dest = encode_register(lhs);
                let from = encode_register(rhs);
                bytecode.push(dest);
                bytecode.push(from);
            }
            InstructionKind::LitMem => {
                let lhs = instruction.lhs();
                let rhs = instruction.rhs();
                let (lower, upper) = encode_literal_or_address(lhs, &labels);
                bytecode.push(lower);
                bytecode.push(upper);
                let (lower, upper) = encode_literal_or_address(rhs, &labels);
                bytecode.push(lower);
                bytecode.push(upper);
            }
            InstructionKind::SingleReg => {
                let lhs = instruction.lhs();
                let register = encode_register(lhs);
                bytecode.push(register);
            }
            InstructionKind::SingleLit => {
                let lhs = instruction.lhs();
                let (lower, upper) = encode_literal_or_address(lhs, &labels);
                bytecode.push(lower);
                bytecode.push(upper);
            }
            InstructionKind::NoArgs => {}
            InstructionKind::Nop => unreachable!(),
        }
    }

    bytecode
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
