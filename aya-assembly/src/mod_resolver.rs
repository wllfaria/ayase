use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::parser::ast::{Ast, Statement};

#[derive(Debug)]
pub enum Either {
    ResolvedValue(u16),
    ModuleField { module: String, field: String },
}

#[derive(Debug)]
pub struct ResolvedModule {
    pub name: String,
    pub path: PathBuf,
    pub imports: Vec<PathBuf>,
    pub symbols: HashMap<String, u16>,
    pub variables: Option<HashMap<String, Either>>,
}

#[derive(Debug)]
pub struct ResolvedModules {
    pub modules: Vec<ResolvedModule>,
    pub sources: HashMap<PathBuf, String>,
    pub asts: Vec<Ast>,
}

pub struct ResolvedModulesIntoIter {
    iter: std::vec::IntoIter<(ResolvedModule, String, Ast)>,
}

impl IntoIterator for ResolvedModules {
    type IntoIter = ResolvedModulesIntoIter;
    type Item = (ResolvedModule, String, Ast);

    fn into_iter(mut self) -> Self::IntoIter {
        let iter = self
            .modules
            .into_iter()
            .zip(self.asts)
            .map(|(module, ast)| {
                let source = self.sources.remove(&module.path).unwrap();
                (module, source, ast)
            })
            .collect::<Vec<_>>()
            .into_iter();

        ResolvedModulesIntoIter { iter }
    }
}

impl Iterator for ResolvedModulesIntoIter {
    type Item = (ResolvedModule, String, Ast);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub fn resolve<P: AsRef<Path>>(code: String, path: P) -> miette::Result<ResolvedModules> {
    let path = path.as_ref().to_path_buf();
    let mut context = Context {
        asts: vec![],
        modules: vec![],
        visited: HashSet::default(),
        sources: HashMap::default(),
    };

    resolve_module("main", path.clone(), code, None, &mut context)?;

    Ok(ResolvedModules {
        sources: context.sources,
        asts: context.asts,
        modules: context.modules,
    })
}

struct Context {
    asts: Vec<Ast>,
    modules: Vec<ResolvedModule>,
    visited: HashSet<PathBuf>,
    sources: HashMap<PathBuf, String>,
}

fn resolve_module(
    name: &str,
    path: PathBuf,
    code: String,
    variables: Option<HashMap<String, Either>>,
    context: &mut Context,
) -> miette::Result<()> {
    if context.visited.contains(&path) {
        return Ok(());
    }
    context.visited.insert(path.clone());

    let ast = crate::parser::parse(&code).expect("failed to parse");

    let mut module = ResolvedModule {
        name: name.to_string(),
        path: path.clone(),
        variables,
        symbols: Default::default(),
        imports: Default::default(),
    };

    resolve_constants(&code, &mut module, &ast)?;
    resolve_imports(&code, &mut module, &ast, context)?;

    context.asts.push(ast);
    context.sources.insert(path, code);
    context.modules.push(module);

    Ok(())
}

fn resolve_constants(code: &str, module: &mut ResolvedModule, ast: &Ast) -> miette::Result<()> {
    for (name, value, exported) in ast.constants() {
        let Statement::HexLiteral(value) = value else {
            unreachable!();
        };

        let value_str = &code[Range::from(*value)];
        let Ok(value_hex) = u16::from_str_radix(value_str, 16) else {
            let offset = if *exported { 1 } else { 0 };
            let labels = vec![
                miette::LabeledSpan::at(*value, "this value"),
                miette::LabeledSpan::at(name.start - offset..value.end, "this constant"),
            ];
            return Err(bail_multi(
                code,
                labels,
                "[INVALID_CONSTANT]: error while resolving constant",
                "hex number is not within the u16 range",
            ));
        };

        let name = &code[Range::from(*name)];
        module.symbols.insert(name.to_string(), value_hex);
    }

    Ok(())
}

fn resolve_imports(code: &str, module: &mut ResolvedModule, ast: &Ast, context: &mut Context) -> miette::Result<()> {
    for (name, path, variables, _) in ast.imports() {
        let variables = resolve_import_vars(code, module, variables)?;
        let name = &code[name.start..name.end];
        let path = &code[path.start..path.end];
        let code = crate::file::load_module_from_path(path).unwrap();
        resolve_module(name, path.into(), code, Some(variables), context)?;
        module.imports.push(path.into());
    }
    Ok(())
}

fn resolve_import_vars(
    code: &str,
    module: &mut ResolvedModule,
    variables: &[Statement],
) -> miette::Result<HashMap<String, Either>> {
    let mut resolved_variables = HashMap::default();

    for variable in variables {
        let Statement::ImportVar { name, value } = variable else {
            unreachable!();
        };

        let name_str = &code[Range::from(*name)];
        if resolved_variables.contains_key(name_str) {
            return Err(bail(
                code,
                variable.offset(),
                "[DUPLICATE_VARIABLE] this variables was previously defined",
                "variable names must be unique",
            ));
        }

        match value.as_ref() {
            Statement::Var(offset) => {
                let var = &code[Range::from(*offset)];
                let Some(value) = module.symbols.get(var) else {
                    return Err(bail(
                        code,
                        variable.offset(),
                        "[UNDEFINED_VARIABLE] this variables doesn't exist in the current scope",
                        "import variables must reference constants",
                    ));
                };
                resolved_variables.insert(name_str.into(), Either::ResolvedValue(*value));
            }
            Statement::HexLiteral(offset) => {
                let value = &code[Range::from(*offset)];
                let Ok(value_hex) = u16::from_str_radix(value, 16) else {
                    let labels = vec![
                        miette::LabeledSpan::at(variable.offset(), "this variable"),
                        miette::LabeledSpan::at(*offset, "this value"),
                    ];
                    return Err(bail_multi(
                        code,
                        labels,
                        "[INVALID_CONSTANT]: error while resolving constant",
                        "hex number is not within the u16 range",
                    ));
                };

                resolved_variables.insert(name_str.to_string(), Either::ResolvedValue(value_hex));
            }
            Statement::FieldAccessor { module, field } => {
                let module = &code[Range::from(*module)];
                let field = &code[Range::from(*field)];
                resolved_variables.insert(
                    name_str.to_string(),
                    Either::ModuleField {
                        module: module.into(),
                        field: field.into(),
                    },
                );
            }
            _ => unreachable!(),
        }
    }

    Ok(resolved_variables)
}

fn bail_multi<S: AsRef<str>>(
    source: &str,
    labels: impl IntoIterator<Item = miette::LabeledSpan>,
    message: S,
    help: S,
) -> miette::Error {
    miette::Error::from(
        miette::MietteDiagnostic::new(message.as_ref())
            .with_labels(labels)
            .with_help(help.as_ref()),
    )
    .with_source_code(source.to_string())
}

fn bail<S: AsRef<str>>(source: &str, span: impl Into<miette::SourceSpan>, message: S, help: S) -> miette::Error {
    miette::Error::from(
        miette::MietteDiagnostic::new(message.as_ref())
            .with_label(miette::LabeledSpan::at(span, "this bit"))
            .with_help(help.as_ref()),
    )
    .with_source_code(source.to_string())
}
