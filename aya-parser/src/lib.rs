mod common;
mod expressions;
mod formats;
mod instructions;
mod types;

use common::label;
use instructions::parse_instruction;
pub use instructions::{Atom, Instruction, InstructionKind};
use nom::branch::alt;
use nom::combinator::map;
use nom::multi::many1;

pub fn parse(input: &str) -> Vec<Instruction> {
    let (input, result) = many1(alt((parse_instruction, map(label, Instruction::Nop))))(input).expect("failed");
    println!("'{input}'");
    assert!(input.is_empty());
    result
}
