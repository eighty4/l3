pub(crate) mod node;
pub(crate) mod typescript;

#[cfg(test)]
mod node_test;
#[cfg(test)]
mod typescript_test;

use crate::code::source::path::SourcePath;
use crate::code::source::ModuleImport;

pub trait ImportResolver: Send + Sync {
    fn resolve(&self, from: &SourcePath, import: &str) -> ModuleImport;
}
