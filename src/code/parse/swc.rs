use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use swc::config::IsModule;
use swc::try_with_handler;
use swc_common::{SourceMap, GLOBALS};
use swc_ecma_ast::{Decl, EsVersion, Module, ModuleDecl, Program};
use swc_ecma_parser::Syntax;

use crate::code::source::{Language, SourceFile};

struct EsModule {
    language: Language,
    module: Module,
    path: PathBuf,
}

pub fn parse_source_file(
    language: Language,
    path: PathBuf,
    project_dir: &Path,
) -> Result<SourceFile, anyhow::Error> {
    debug_assert!(project_dir.is_absolute());
    debug_assert!(path.is_relative());
    debug_assert!(language != Language::Python);
    Ok(SourceFile::from(parse_module_ast(
        language,
        project_dir.join(&path),
    )?))
}

impl From<EsModule> for SourceFile {
    fn from(module: EsModule) -> Self {
        let mut exported_fns: Vec<String> = Vec::new();
        let mut imports: Vec<PathBuf> = Vec::new();
        for module_item in module.module.body {
            if let Some(module_decl) = module_item.module_decl() {
                match module_decl {
                    ModuleDecl::Import(import_decl) => {
                        imports.push(PathBuf::from(import_decl.src.value.as_str()));
                    }
                    ModuleDecl::ExportDecl(export_decl) => match export_decl.decl {
                        Decl::Fn(func) => {
                            exported_fns.push(parse_identifier_name(func.ident.as_ref()))
                        }
                        Decl::Var(var_decl) => {
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
                    },
                    _ => {}
                }
            };
        }
        SourceFile {
            imports,
            exported_fns,
            language: module.language,
            path: module.path,
        }
    }
}

fn parse_identifier_name(n: &str) -> String {
    n.trim_end_matches(char::is_numeric)
        .trim_end_matches(char::is_numeric)
        .trim_end_matches('#')
        .to_string()
}

fn parse_module_ast(language: Language, path: PathBuf) -> Result<EsModule, anyhow::Error> {
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
                    source_map.load_file(&path)?,
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
        Program::Module(module) => Ok(EsModule {
            language,
            module,
            path,
        }),
        Program::Script(_) => Err(anyhow!(
            "unable to parse CJS format for source file {}",
            path.to_string_lossy()
        )),
    }
}
