use crate::result::{FnBuildError, FnSource, ModuleImport};
use crate::swc::compiler::SwcCompiler;
use crate::swc::visitors::ImportVisitor;
use std::path::PathBuf;
use swc_ecma_visit::FoldWith;

mod compiler;
mod visitors;

#[cfg(test)]
mod visitors_test;

pub async fn parse_js_fn(path: PathBuf) -> Result<FnSource, FnBuildError> {
    let compiler = SwcCompiler::new();
    let module = compiler.parse_es_module(&path).unwrap();
    let mut visitor = ImportVisitor::new();
    module.fold_with(&mut visitor);
    let imports = visitor
        .result()
        .into_iter()
        .map(ModuleImport::Unknown)
        .collect();
    Ok(FnSource { imports, path })
}
