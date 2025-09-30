use crate::runtime::node::NodeConfig;
use crate::{FnHandler, FnParseResult, FnSource, ModuleImport};
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod build_fn;
pub mod node;
mod parse_fn;
pub mod python;
mod ts_imports;

#[cfg(test)]
mod build_fn_test;

#[cfg(test)]
mod ts_imports_test;

trait ImportResolver: Send + Sync {
    fn resolve(&self, project_dir: &Path, from: &Path, import: &str) -> ModuleImport;
}

trait FnSourceParser: Send + Sync {
    /// Parse a source file for exported functions that can be bound as a FnHandler.
    fn collect_handlers(
        &self,
        project_dir: &Path,
        source_path: &Path,
    ) -> FnParseResult<Vec<FnHandler>>;
    /// Provide sources for deployment to a serverless runtime that are not explicit imports of the
    /// function's source tree. For Node.js this would include package.json.
    fn collect_runtime_sources(&self, project_dir: &Path) -> Vec<FnSource>;
    /// Parse a function entrypoint for exports that could be deployed as lambda functions.
    fn parse_fn_entrypoint(
        &self,
        project_dir: &Path,
        source_path: PathBuf,
    ) -> FnParseResult<(FnSource, Vec<FnHandler>)>;
    /// Parse a source file to find imported sources for a function's source tree.
    fn parse_for_imports(
        &self,
        project_dir: &Path,
        source_path: PathBuf,
    ) -> FnParseResult<FnSource>;
}

#[derive(Clone)]
pub enum Runtime {
    /// A Node.js build. `package.json` will be parsed from the project directory unless
    /// NodeConfig is explicitly provided.
    Node(Option<Arc<NodeConfig>>),
    Python,
}
