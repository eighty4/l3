use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::code::runtime::RuntimeConfig;
use crate::code::source::path::SourcePath;
use crate::code::source::{Language, ModuleImport, SourceFile};
use anyhow::anyhow;
use swc::config::IsModule;
use swc::try_with_handler;
use swc_common::{SourceMap, GLOBALS};
use swc_ecma_ast::{Decl, EsVersion, ExportDecl, Module, ModuleDecl, Program};
use swc_ecma_parser::Syntax;

pub fn parse_source_file(
    language: Language,
    path: SourcePath,
    runtime_config: Arc<Mutex<RuntimeConfig>>,
) -> Result<SourceFile, anyhow::Error> {
    debug_assert!(
        matches!(language, Language::JavaScript) || matches!(language, Language::TypeScript)
    );
    let module = parse_module_ast(&language, &path)?;
    Ok(build_source_file_from_module_ast(language, module, path))
}

fn parse_module_ast(language: &Language, path: &SourcePath) -> Result<Module, anyhow::Error> {
    let source_map = Arc::<SourceMap>::default();
    let compiler = swc::Compiler::new(source_map.clone());
    let program = GLOBALS
        .set(&Default::default(), || {
            try_with_handler(source_map.clone(), Default::default(), |handler| {
                let syntax = match language {
                    Language::JavaScript => Syntax::Es(Default::default()),
                    Language::TypeScript => Syntax::Typescript(Default::default()),
                    _ => panic!(),
                };
                compiler.parse_js(
                    source_map.load_file(&path.abs)?,
                    handler,
                    EsVersion::EsNext,
                    syntax,
                    IsModule::Unknown,
                    None,
                )
            })
        })
        .map_err(|err| anyhow!("error from compiler parsing JS: {}", err.to_string()))?;
    match program {
        Program::Module(module) => Ok(module),
        Program::Script(_) => Err(anyhow!(
            "unable to parse CJS format for source file {}",
            path.rel.to_string_lossy()
        )),
    }
}

fn build_source_file_from_module_ast(
    language: Language,
    module: Module,
    path: SourcePath,
) -> SourceFile {
    let mut exported_fns: Vec<String> = Vec::new();
    let mut imports: Vec<ModuleImport> = Vec::new();
    for module_item in module.body {
        if let Some(module_decl) = module_item.module_decl() {
            match module_decl {
                ModuleDecl::Import(import_decl) => {
                    imports.push(parse_import_path(import_decl.src.value.as_str(), &path))
                }
                ModuleDecl::ExportDecl(ExportDecl {
                    decl: Decl::Fn(fn_decl),
                    ..
                }) => exported_fns.push(parse_identifier_name(fn_decl.ident.as_ref())),
                ModuleDecl::ExportDecl(ExportDecl {
                    decl: Decl::Var(var_decl),
                    ..
                }) => {
                    for var_declarator in var_decl.decls {
                        if let Some(expr) = var_declarator.init {
                            if expr.as_arrow().is_some() {
                                exported_fns.push(parse_identifier_name(
                                    var_declarator.name.ident().unwrap().as_ref(),
                                ))
                            }
                        }
                    }
                }
                _ => {}
            }
        };
    }
    SourceFile::new(exported_fns, imports, language, path)
}

// todo resolve ts and js import path mappings
//  https://nodejs.org/api/packages.html#subpath-imports
//  https://www.typescriptlang.org/tsconfig/#paths
//  https://www.typescriptlang.org/docs/handbook/modules/reference.html#paths
//  https://github.com/swc-project/swc/blob/95af2536a2cd5040f44e93f2eea9cf577558f335/crates/swc_ecma_loader/src/resolvers/node.rs
fn parse_import_path(import_path: &str, source_path: &SourcePath) -> ModuleImport {
    if import_path.starts_with('.') {
        ModuleImport::RelativeSource(source_path.to_relative_source(&PathBuf::from(import_path)))
    } else {
        ModuleImport::Unknown(import_path.to_string())
    }

    /*

    if import_path.starts_with('.') {
        ModuleImport::RelativeSource(source_path.to_relative_source(&PathBuf::from(import_path)))
    } else {
        let (maybe_dep_package, maybe_dep_subpath) = match import_path.split_once('/') {
            None => (import_path.to_string(), None),
            Some((base, remainder)) => (base.to_string(), Some(remainder.to_string())),
        };
        if project_details
            .javascript
            .has_dependency(&maybe_dep_package)
        {
            ModuleImport::PackageDependency {
                package: maybe_dep_package,
                subpath: maybe_dep_subpath,
            }
        } else {
            // todo resolve ts path alias
            ModuleImport::Unknown(import_path.to_string())
        }
    }

    */
}

fn parse_identifier_name(n: &str) -> String {
    n.trim_end_matches(char::is_numeric)
        .trim_end_matches(char::is_numeric)
        .trim_end_matches('#')
        .to_string()
}
