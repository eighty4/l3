use crate::paths::join_relative_path;
use crate::result::ModuleImport;
use std::path::{Path, PathBuf};

pub trait ImportResolver: Send + Sync {
    fn resolve(&self, project_dir: &Path, from: &Path, import: &str) -> ModuleImport;
}

// todo nodejs subpath imports
//  https://nodejs.org/api/packages.html#subpath-imports
// todo cross-check swc implementation
//  https://github.com/swc-project/swc/blob/main/crates/swc_ecma_loader/src/resolvers/node.rs
pub struct NodeImportResolver {}

impl ImportResolver for NodeImportResolver {
    fn resolve(&self, project_dir: &Path, from: &Path, import: &str) -> ModuleImport {
        if import.starts_with('.') {
            let maybe = project_dir.join(join_relative_path(from, &PathBuf::from(import)));
            if maybe.is_file() {
                return ModuleImport::RelativeSource(
                    maybe.strip_prefix(project_dir).unwrap().to_path_buf(),
                );
            }
            println!("{}", maybe.to_string_lossy());
        }
        ModuleImport::Unknown(import.to_string())
    }
}
