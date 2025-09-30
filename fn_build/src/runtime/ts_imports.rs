use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::paths::join_file_paths;
use crate::runtime::ImportResolver;
use crate::typescript::TsConfigJson;
use crate::ModuleImport;

// todo check ts path aliases
pub struct TypeScriptImportResolver {
    runtime: Box<dyn ImportResolver>,
    #[allow(dead_code)]
    tsconfig: Arc<TsConfigJson>,
}

impl TypeScriptImportResolver {
    pub fn new(tsconfig: Arc<TsConfigJson>, runtime: Box<dyn ImportResolver>) -> Self {
        Self { tsconfig, runtime }
    }

    fn resolve_relative_path(
        &self,
        project_dir: &Path,
        from: &Path,
        import: &str,
    ) -> Option<PathBuf> {
        debug_assert!(import.starts_with('.'));
        let path = join_file_paths(from, &PathBuf::from(import)).with_extension("ts");
        if project_dir.join(&path).is_file() {
            Some(path)
        } else {
            None
        }
    }
}

impl ImportResolver for TypeScriptImportResolver {
    fn resolve(&self, project_dir: &Path, from: &Path, import: &str) -> ModuleImport {
        if import.starts_with('.') {
            if let Some(resolved) = self.resolve_relative_path(project_dir, from, import) {
                return ModuleImport::RelativeSource(resolved);
            }
        }
        self.runtime.resolve(project_dir, from, import)
    }
}
