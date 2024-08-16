use crate::code::parse::imports::ImportResolver;
use crate::code::runtime::node::NodeConfig;
use crate::code::source::path::SourcePath;
use crate::code::source::ModuleImport;
use std::path::PathBuf;
use std::sync::Arc;

// todo nodejs subpath imports
//  https://nodejs.org/api/packages.html#subpath-imports
// todo cross-check swc implementation
//  https://github.com/swc-project/swc/blob/main/crates/swc_ecma_loader/src/resolvers/node.rs
pub struct NodeImportResolver {
    node_config: Arc<NodeConfig>,
}

impl NodeImportResolver {
    pub fn new(node_config: Arc<NodeConfig>) -> Self {
        Self { node_config }
    }
}

impl ImportResolver for NodeImportResolver {
    fn resolve(&self, from: &SourcePath, import: &str) -> ModuleImport {
        if import.starts_with('.') {
            ModuleImport::RelativeSource(from.to_relative_source(&PathBuf::from(import)))
        } else {
            ModuleImport::Unknown(import.to_string())
        }
    }
}
