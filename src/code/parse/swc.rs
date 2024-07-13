use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use swc::config::IsModule;
use swc::try_with_handler;
use swc_common::{SourceMap, GLOBALS};
use swc_ecma_ast::{Decl, EsVersion, ExportDecl, Module, ModuleDecl, Program};
use swc_ecma_parser::Syntax;

use crate::code::project::ProjectDetails;
use crate::code::source::path::SourcePath;
use crate::code::source::{Language, ModuleImport, SourceFile};

pub fn parse_source_file(
    language: Language,
    path: SourcePath,
    project_details: &ProjectDetails,
) -> Result<SourceFile, anyhow::Error> {
    debug_assert!(language == Language::JavaScript || language == Language::TypeScript);
    let module = parse_module_ast(&language, &path)?;
    Ok(build_source_file_from_module_ast(
        language,
        module,
        path,
        project_details,
    ))
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
    project_details: &ProjectDetails,
) -> SourceFile {
    let mut exported_fns: Vec<String> = Vec::new();
    let mut imports: Vec<ModuleImport> = Vec::new();
    for module_item in module.body {
        if let Some(module_decl) = module_item.module_decl() {
            match module_decl {
                ModuleDecl::Import(import_decl) => imports.push(parse_import_path(
                    import_decl.src.value.as_str(),
                    &path,
                    project_details,
                )),
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

fn parse_import_path(
    import_path: &str,
    source_path: &SourcePath,
    project_details: &ProjectDetails,
) -> ModuleImport {
    if import_path.starts_with('.') {
        ModuleImport::RelativeSource(source_path.to_relative_source(&PathBuf::from(import_path)))
    } else if import_path.chars().next().map_or(false, |c| c == '#') {
        match project_details.javascript.map_subpath_import(import_path) {
            None => ModuleImport::Unknown(import_path.to_string()),
            Some(p) => ModuleImport::NodeSubpathImport {
                declared: import_path.to_string(),
                path: SourcePath::from_rel(&project_details.project_dir, p),
            },
        }
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
}

fn parse_identifier_name(n: &str) -> String {
    n.trim_end_matches(char::is_numeric)
        .trim_end_matches(char::is_numeric)
        .trim_end_matches('#')
        .to_string()
}
