pub(crate) mod node;
pub(crate) mod typescript;

use crate::code::source::path::SourcePath;
use crate::code::source::ModuleImport;

pub trait ImportResolver: Send + Sync {
    fn resolve(&self, from: &SourcePath, import: &str) -> ModuleImport;
}
