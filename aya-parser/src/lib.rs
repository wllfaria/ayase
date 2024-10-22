mod ast;
mod lexer;
mod parser;

pub use ast::{Ast, Instruction, InstructionKind, Statement};
pub use parser::parse;
