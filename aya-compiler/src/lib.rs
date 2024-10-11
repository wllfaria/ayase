use aya_core::register::Register;
use aya_parser::{Atom, Instruction, InstructionKind};

fn encode_literal_or_address<'parser>(value: &'parser Atom<'parser>) -> (u8, u8) {
    let value = match value {
        Atom::HexLiteral(value) => value,
        Atom::Address(value) => value,
        _ => unreachable!(),
    };

    let Ok(hex) = u16::from_str_radix(value, 16) else {
        panic!("hex value is within an invalid range");
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

pub fn compile(source: &str) -> Vec<u8> {
    let result = aya_parser::parse(source);
    let mut bytecode = vec![];

    for instruction in result {
        bytecode.push(instruction.opcode().into());

        match instruction.kind() {
            InstructionKind::LitReg | InstructionKind::MemReg => {
                let lhs = instruction.lhs();
                let rhs = instruction.rhs();
                let register = encode_register(lhs);
                let (lower, upper) = encode_literal_or_address(rhs);
                bytecode.push(register);
                bytecode.push(lower);
                bytecode.push(upper);
            }
            InstructionKind::RegLit | InstructionKind::RegMem => {
                let lhs = instruction.lhs();
                let rhs = instruction.rhs();
                let (lower, upper) = encode_literal_or_address(lhs);
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
                let (lower, upper) = encode_literal_or_address(lhs);
                bytecode.push(lower);
                bytecode.push(upper);
                let (lower, upper) = encode_literal_or_address(rhs);
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
                let (lower, upper) = encode_literal_or_address(lhs);
                bytecode.push(lower);
                bytecode.push(upper);
            }
            InstructionKind::NoArgs => {}
        }
    }

    bytecode
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample() {
        let program = r#"
mov r1, $4200
mov &0060, r1
mov r1, $1300
mov r2, &0060
add r1, r2
        "#;

        let result = compile(program);
        println!("{result:02X?}");

        panic!();
    }
}
