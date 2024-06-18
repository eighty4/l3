use std::path::Path;
use std::sync::Arc;

use swc::config::IsModule;
use swc::try_with_handler;
use swc_common::{SourceMap, GLOBALS};
use swc_ecma_ast::{Decl, EsVersion, ModuleDecl, ModuleItem, Program};
use swc_ecma_parser::{EsConfig, Syntax};

pub fn parse_module_for_exported_fns(path: &Path) -> Vec<String> {
    let program = parse_module_for_ast(path);
    let mut exported_fns = Vec::new();
    match program {
        Program::Module(module) => {
            for module_item in module.body {
                match module_item {
                    ModuleItem::ModuleDecl(module_decl) => {
                        if let ModuleDecl::ExportDecl(export) = module_decl {
                            if let Decl::Fn(func) = export.decl {
                                exported_fns.push(
                                    func.ident
                                        .to_string()
                                        .trim_end_matches(char::is_numeric)
                                        .trim_end_matches(char::is_numeric)
                                        .trim_end_matches('#')
                                        .to_string(),
                                )
                            }
                        }
                    }
                    ModuleItem::Stmt(_) => {}
                }
            }
        }
        Program::Script(_) => panic!(
            "{} should be an ES module",
            path.file_name().unwrap().to_string_lossy()
        ),
    }
    exported_fns
}

fn parse_module_for_ast(path: &Path) -> Program {
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
        .unwrap()
}
