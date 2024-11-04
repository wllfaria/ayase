mod codegen;
mod compiler;
mod file;
mod lexer;
mod mod_resolver;
mod parser;
mod utils;

use std::path::Path;

pub use codegen::generate;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum AssembleBehavior {
    Bytecode,
    Codegen,
}

#[derive(Debug)]
pub enum AssembleOutput {
    Bytecode(Vec<u8>),
    Codegen(String),
}

pub fn assemble<P: AsRef<Path>>(path: P, behavior: AssembleBehavior) -> miette::Result<AssembleOutput> {
    let code = file::load_module_from_path(&path).unwrap();
    let modules = mod_resolver::resolve(code, &path)?;
    let modules = codegen::generate(modules)?;

    match behavior {
        AssembleBehavior::Codegen => Ok(AssembleOutput::Codegen(modules.iter().fold(
            String::default(),
            |mut acc, m| {
                if !m.code.is_empty() {
                    acc.push_str(&m.code);
                    acc.push('\n');
                }
                acc
            },
        ))),
        AssembleBehavior::Bytecode => Ok(AssembleOutput::Bytecode(compiler::compile(modules)?)),
    }
}
