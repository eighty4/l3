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
    _node_config: Arc<NodeConfig>,
}

impl NodeImportResolver {
    pub fn new(_node_config: Arc<NodeConfig>) -> Self {
        Self { _node_config }
    }
}

impl ImportResolver for NodeImportResolver {
    fn resolve(&self, from: &SourcePath, import: &str) -> ModuleImport {
        if import.starts_with('.') {
            let maybe_source_path = from.to_relative_source(&PathBuf::from(import));
            if maybe_source_path.rel.extension().is_some() && maybe_source_path.abs.is_file() {
                return ModuleImport::RelativeSource(maybe_source_path);
            }
        }
        ModuleImport::Unknown(import.to_string())
    }
}
