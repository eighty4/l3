use crate::code::parse::imports::ImportResolver;
use crate::code::source::path::SourcePath;
use crate::code::source::ModuleImport;

pub struct TypescriptImportResolver {}

impl TypescriptImportResolver {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImportResolver for TypescriptImportResolver {
    fn resolve(&self, from: &SourcePath, import: String) -> ModuleImport {
        todo!()
    }
}
