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

pub fn collect_labels<'assembler>(ast: &'assembler [Instruction<'assembler>]) -> HashMap<&'assembler str, u16> {
    let mut labels: HashMap<&str, u16> = HashMap::default();
    let mut current_address = 0;

    for instruction in ast {
        if let Instruction::Nop(Atom::Label(name)) = instruction {
            labels.insert(name, current_address);
            continue;
        }
        current_address += instruction.kind().byte_size() as u16;
    }

    labels
}

pub fn compile(source: &str) -> Vec<u8> {
    let ast = aya_parser::parse(source);

    let labels = collect_labels(&ast);

    let mut bytecode = vec![];

    for instruction in ast.iter() {
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
}
