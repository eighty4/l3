use crate::code::parse::imports::ImportResolver;
use crate::code::source::path::SourcePath;
use crate::code::source::ModuleImport;

// todo ts paths
//  https://www.typescriptlang.org/tsconfig/#paths
//  https://www.typescriptlang.org/docs/handbook/modules/reference.html#paths
// todo delegate to NodeImportResolver for non-ts path relative source imports
pub struct TypescriptImportResolver {}

impl TypescriptImportResolver {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImportResolver for TypescriptImportResolver {
    fn resolve(&self, from: &SourcePath, import: &str) -> ModuleImport {
        todo!()
    }
}
