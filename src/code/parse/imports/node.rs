use crate::code::parse::imports::ImportResolver;
use crate::code::runtime::node::NodeConfig;
use crate::code::source::path::SourcePath;
use crate::code::source::ModuleImport;
use std::sync::Arc;

pub struct NodeImportResolver {
    node_config: Arc<NodeConfig>,
}

impl NodeImportResolver {
    pub fn new(node_config: Arc<NodeConfig>) -> Self {
        Self { node_config }
    }
}

impl ImportResolver for NodeImportResolver {
    fn resolve(&self, from: &SourcePath, import: String) -> ModuleImport {
        todo!()
    }
}
