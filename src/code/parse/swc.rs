use crate::code::parse::SourceParser;
use crate::code::source::path::SourcePath;
use crate::code::source::{Language, ModuleImports, SourceFile};
use crate::code::swc::compiler::with_swc_compiler_without_diagnostics;
use anyhow::anyhow;
use swc::config::IsModule;
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
    with_swc_compiler_without_diagnostics::<_, Module>(|compiler, handler, source_map| {
        let syntax = match language {
            Language::JavaScript => Syntax::Es(Default::default()),
            Language::TypeScript => Syntax::Typescript(Default::default()),
            _ => panic!(),
        };
        let program = compiler.parse_js(
            source_map.load_file(&path.abs)?,
            handler,
            EsVersion::EsNext,
            syntax,
            IsModule::Unknown,
            None,
        )?;
        match program {
            Program::Module(module) => Ok(module),
            Program::Script(_) => Err(anyhow!(
                "L3 does not support CJS format used in source file {}",
                path.rel.to_string_lossy()
            )),
        }
    })
}

// todo use visitor pattern to find import declarations in classes and fns
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
                            if expr.as_arrow().is_some() || expr.as_fn_expr().is_some() {
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
