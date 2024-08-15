use std::path::PathBuf;
use std::sync::Arc;

use crate::code::parse::SourceParser;
use crate::code::source::path::SourcePath;
use crate::code::source::{Language, ModuleImport, ModuleImports, SourceFile};
use anyhow::anyhow;
use swc::config::IsModule;
use swc::try_with_handler;
use swc_common::{SourceMap, GLOBALS};
use swc_ecma_ast::{Decl, EsVersion, ExportDecl, Module, ModuleDecl, Program};
use swc_ecma_parser::Syntax;

pub struct SwcSourceParser {
    language: Language,
}

impl SwcSourceParser {
    pub fn for_javascript() -> Self {
        Self::new(Language::JavaScript)
    }

    pub fn for_typescript() -> Self {
        Self::new(Language::TypeScript)
    }

    fn new(language: Language) -> Self {
        Self { language }
    }
}

impl SourceParser for SwcSourceParser {
    fn parse(&self, path: SourcePath) -> Result<SourceFile, anyhow::Error> {
        let (exported_fns, imports) = parse_ast(create_ast(&self.language, &path)?);
        Ok(SourceFile::new(
            exported_fns,
            if imports.is_empty() {
                ModuleImports::Empty
            } else {
                ModuleImports::Unprocessed(imports)
            },
            self.language.clone(),
            path,
        ))
    }
}

fn create_ast(language: &Language, path: &SourcePath) -> Result<Module, anyhow::Error> {
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

fn parse_ast(module: Module) -> (Vec<String>, Vec<String>) {
    let mut exported_fns: Vec<String> = Vec::new();
    let mut imports: Vec<String> = Vec::new();
    for module_item in module.body {
        if let Some(module_decl) = module_item.module_decl() {
            match module_decl {
                ModuleDecl::Import(import_decl) => imports.push(import_decl.src.value.to_string()),
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
    (exported_fns, imports)
}

fn parse_identifier_name(n: &str) -> String {
    n.trim_end_matches(char::is_numeric)
        .trim_end_matches(char::is_numeric)
        .trim_end_matches('#')
        .to_string()
}
