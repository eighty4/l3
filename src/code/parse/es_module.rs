use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use swc::config::IsModule;
use swc::try_with_handler;
use swc_common::{SourceMap, GLOBALS};
use swc_ecma_ast::{Decl, EsVersion, ModuleDecl, Program};
use swc_ecma_parser::{EsConfig, Syntax};

pub struct EsModule {
    pub exported_fns: Vec<String>,
    #[allow(unused)]
    pub module_imports: Vec<PathBuf>,
}

impl EsModule {
    pub fn parse(path: &Path) -> Result<Self, anyhow::Error> {
        let program = parse_module_for_ast(path)?;
        let mut exported_fns: Vec<String> = Vec::new();
        let mut module_imports: Vec<PathBuf> = Vec::new();
        match program {
            Program::Module(module) => {
                for module_item in module.body {
                    if let Some(module_decl) = module_item.module_decl() {
                        match module_decl {
                            ModuleDecl::Import(import_decl) => {
                                module_imports.push(PathBuf::from(import_decl.src.value.as_str()));
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
            }
            Program::Script(_) => {
                return Err(anyhow!(
                    "unable to parse CJS format for source file {}",
                    path.to_string_lossy()
                ))
            }
        }
        Ok(Self {
            module_imports,
            exported_fns,
        })
    }
}

fn parse_identifier_name(n: &str) -> String {
    n.trim_end_matches(char::is_numeric)
        .trim_end_matches(char::is_numeric)
        .trim_end_matches('#')
        .to_string()
}

fn parse_module_for_ast(path: &Path) -> Result<Program, anyhow::Error> {
    let source_map = Arc::<SourceMap>::default();
    let compiler = swc::Compiler::new(source_map.clone());
    GLOBALS
        .set(&Default::default(), || {
            try_with_handler(source_map.clone(), Default::default(), |handler| {
                compiler.parse_js(
                    source_map.load_file(path)?,
                    handler,
                    EsVersion::EsNext,
                    Syntax::Es(EsConfig::default()),
                    IsModule::Unknown,
                    None,
                )
            })
        })
        .map_err(|err| anyhow!("error from compiler parsing JS: {}", err.to_string()))
}
