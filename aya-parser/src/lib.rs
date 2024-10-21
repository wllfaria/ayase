mod ast;
mod lexer;
mod parser;

pub use ast::{Ast, InstructionKind, Statement};
pub use parser::parse;
