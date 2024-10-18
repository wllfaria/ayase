mod common;
mod expressions;
mod formats;
mod instructions;
mod modules;
mod types;

use common::{constant, data, label};
use instructions::parse_instruction;
pub use instructions::{Ast, Instruction, InstructionKind};
use nom::branch::alt;
use nom::combinator::map;
use nom::multi::many1;

pub fn parse(input: &str) -> Vec<Ast> {
    let (input, result) = many1(alt((
        label,
        data,
        constant,
        map(parse_instruction, |instr| Ast::Instruction(Box::new(instr))),
    )))(input)
    .expect("failed");

    println!("'{input}'");
    assert!(input.is_empty());
    result
}
