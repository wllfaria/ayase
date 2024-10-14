mod common;
mod expressions;
mod formats;
mod instructions;
mod types;

use common::{constant, data, label};
use instructions::parse_instruction;
pub use instructions::{Atom, Instruction, InstructionKind};
use nom::branch::alt;
use nom::combinator::map;
use nom::multi::many1;

pub fn parse(input: &str) -> Vec<Instruction> {
    let (input, result) = many1(alt((
        map(label, Instruction::Nop),
        map(data, Instruction::Nop),
        map(constant, Instruction::Nop),
        parse_instruction,
    )))(input)
    .expect("failed");

    println!("'{input}'");
    assert!(input.is_empty());
    result
}
