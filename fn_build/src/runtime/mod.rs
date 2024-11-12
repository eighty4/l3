use crate::runtime::node::NodeConfig;
use crate::{FnBuildResult, FnSource, ModuleImport};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub mod node;
mod parse_fn;

trait ImportResolver: Send + Sync {
    fn resolve(&self, project_dir: &Path, from: &Path, import: &str) -> ModuleImport;
}

trait FnSourceParser: Send + Sync {
    fn collect_runtime_sources(&self, project_dir: &Path) -> Vec<FnSource>;
    fn parse(&self, project_dir: &Path, source_path: PathBuf) -> FnBuildResult<FnSource>;
}

#[derive(Clone)]
pub enum Runtime {
    Node(Arc<NodeConfig>),
}
