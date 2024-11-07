use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::parser::ast::{Ast, Statement};
use crate::utils::{bail, bail_multi};

#[derive(Debug, Clone)]
pub enum Either {
    ResolvedValue(u16),
    ModuleField { module: String, field: String },
}

impl Either {
    pub fn to_value(&self) -> u16 {
        let Either::ResolvedValue(value) = self else {
            panic!("conversion of Eiter::ModuleField into u16 is not possible");
        };

        *value
    }

    pub fn to_value_small(&self) -> u8 {
        let Either::ResolvedValue(value) = self else {
            panic!("conversion of Eiter::ModuleField into u16 is not possible");
        };

        *value as u8
    }
}

#[derive(Debug, Default, Clone)]
pub struct ResolvedModule {
    pub name: String,
    pub path: PathBuf,
    pub address: u16,
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

    resolve_module("main", path.clone(), code, None, &mut context, 0)?;

    let mut sorted = topological_sort(&context.modules);

    for i in 0..sorted.len() {
        if sorted[i] == usize::MAX {
            continue;
        };
        let mut target = sorted[i];
        let value = std::mem::take(&mut context.modules[i]);
        let ast = std::mem::take(&mut context.asts[i]);
        let mut x = i;
        while i != target {
            sorted[x] = usize::MAX;
            context.modules.swap(x, target);
            context.asts.swap(x, target);
            x = target;
            target = sorted[x];
        }
        context.modules[x] = value;
        context.asts[x] = ast;
        sorted[x] = usize::MAX;
    }

    let symbols: HashMap<(String, String), u16> = context
        .modules
        .iter()
        .flat_map(|module| {
            module
                .symbols
                .iter()
                .map(move |(field, value)| ((module.name.to_string(), field.to_string()), *value))
        })
        .collect();

    for module in context.modules.iter_mut() {
        if let Some(variables) = &mut module.variables {
            for value in variables.values_mut() {
                if let Either::ModuleField { module, field } = value {
                    let new_value = symbols.get(&(module.to_string(), field.to_string())).unwrap();
                    *value = Either::ResolvedValue(*new_value);
                }
            }
        }
    }

    Ok(ResolvedModules {
        sources: context.sources,
        asts: context.asts,
        modules: context.modules,
    })
}

fn topological_sort(modules: &[ResolvedModule]) -> Vec<usize> {
    let mut sorted = Vec::with_capacity(modules.len());
    let mut idx_path = HashMap::with_capacity(modules.len());
    let mut idx_name = HashMap::with_capacity(modules.len());

    for (idx, module) in modules.iter().enumerate() {
        idx_path.insert(&module.path, idx);
        idx_name.insert(&module.name, idx);
    }

    let mut in_degrees = vec![0; modules.len()];
    for module in modules.iter() {
        for import in &module.imports {
            if let Some(&idx) = idx_path.get(import) {
                in_degrees[idx] += 1;
            }
        }

        if let Some(ref variables) = module.variables {
            for value in variables.values() {
                if let Either::ModuleField { module, .. } = value {
                    if let Some(&idx) = idx_name.get(&module) {
                        in_degrees[idx] += 1;
                    }
                }
            }
        }
    }

    let mut queue = VecDeque::new();
    for (index, degree) in in_degrees.iter().enumerate() {
        if *degree == 0 {
            queue.push_back(index);
        }
    }

    while let Some(idx) = queue.pop_front() {
        let module = &modules[idx];
        sorted.push(idx);

        for import in &module.imports {
            if let Some(&idx) = idx_path.get(import) {
                in_degrees[idx] -= 1;
                if in_degrees[idx] == 0 {
                    queue.push_back(idx);
                }
            }
        }

        if let Some(ref variables) = module.variables {
            for value in variables.values() {
                if let Either::ModuleField { module, .. } = value {
                    if let Some(&idx) = idx_name.get(&module) {
                        in_degrees[idx] -= 1;
                        if in_degrees[idx] == 0 {
                            queue.push_back(idx);
                        }
                    }
                }
            }
        }
    }

    if sorted.len() != modules.len() {
        panic!("cyclic dependency, probably");
    }

    sorted.reverse();
    sorted
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
    address: u16,
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
        address,
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
    for (name, path, variables, address) in ast.imports() {
        let variables = resolve_import_vars(code, module, variables)?;
        let name = &code[name.start..name.end];
        let path = &code[path.start..path.end];
        let address = &code[Range::from(*address)];
        let address = u16::from_str_radix(address, 16).unwrap();
        let code = crate::file::load_module_from_path(path).unwrap();
        resolve_module(name, path.into(), code, Some(variables), context, address)?;
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
                "[DUPLICATE_VARIABLE] this variables was previously defined",
                "variable names must be unique",
                variable.offset(),
            ));
        }

        match value.as_ref() {
            // NOTE: if the variable references a var, it can only refence constants.
            Statement::Var(offset) => {
                let var = &code[Range::from(*offset)];
                let Some(value) = module.symbols.get(var) else {
                    return Err(bail(
                        code,
                        "[UNDEFINED_VARIABLE] this variables doesn't exist in the current scope",
                        "import variables must reference constants",
                        variable.offset(),
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
