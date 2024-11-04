use crate::code::parse::imports::ImportResolver;
use crate::code::source::path::SourcePath;
use crate::code::source::ModuleImport;
use std::path::PathBuf;

// todo ts file importing js
// todo ts file importing ts produces build warning
//  because ts will resolve import and compile but will not translate file extension
//  and will error when Node.js runtime does import resolution
// todo ts paths
//  https://www.typescriptlang.org/tsconfig/#paths
//  https://www.typescriptlang.org/docs/handbook/modules/reference.html#paths
pub struct TypescriptImportResolver {}

impl TypescriptImportResolver {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImportResolver for TypescriptImportResolver {
    fn resolve(&self, from: &SourcePath, import: &str) -> ModuleImport {
        if import.starts_with('.') && import.ends_with(".js") {
            let maybe_source_path =
                from.to_relative_source(&PathBuf::from(import).with_extension("ts"));
            if maybe_source_path.abs.is_file() {
                return ModuleImport::RelativeSource(maybe_source_path);
            }
        }
        ModuleImport::Unknown(import.to_string())
    }
}
