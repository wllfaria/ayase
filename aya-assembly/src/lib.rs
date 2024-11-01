mod codegen;
mod compiler;
mod file;
mod lexer;
mod mod_resolver;
mod parser;

use std::path::Path;

pub use codegen::generate;
use codegen::CodegenModule;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum AssembleBehavior {
    Bytecode,
    Codegen,
}

#[derive(Debug)]
pub enum AssembleOutput {
    Bytecode(Vec<u8>),
    Codegen(CodegenModule),
}

pub fn assemble<P: AsRef<Path>>(path: P, behavior: AssembleBehavior) -> miette::Result<AssembleOutput> {
    let code = file::load_module_from_path(&path).unwrap();
    let modules = mod_resolver::resolve(code, &path)?;
    let modules = codegen::generate(modules)?;

    match behavior {
        AssembleBehavior::Codegen => todo!(),
        AssembleBehavior::Bytecode => Ok(AssembleOutput::Bytecode(compiler::compile(modules)?)),
    }
}
