mod common;
mod expressions;
mod formats;
mod instructions;
mod types;

use instructions::parse_instruction;
pub use instructions::{Atom, Instruction, InstructionKind};
use nom::multi::many1;

pub fn parse(input: &str) -> Vec<Instruction> {
    let (input, result) = many1(parse_instruction)(input).expect("failed");
    assert!(input.is_empty());
    result
}
