use crate::result::ModuleImport;
use crate::runtime::node::NodeConfig;
use std::path::Path;
use std::sync::Arc;

pub mod node;

pub trait ImportResolver: Send + Sync {
    fn resolve(&self, project_dir: &Path, from: &Path, import: &str) -> ModuleImport;
}

pub enum Runtime {
    Node(Arc<NodeConfig>),
}
