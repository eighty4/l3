use crate::runtime::node::node_source_parser::NodeFnSourceParser;
use crate::runtime::parse_fn::parse_fn_inner;
use crate::runtime::{FnSourceParser, Runtime};
use crate::{FnBuildResult, FnManifest, FnParseSpec};
pub use build_node_fn::build_node_fn;
pub use imports::resolver::NodeImportResolver;
pub use node_config::*;
use std::sync::Arc;

mod build_node_fn;
pub mod imports;
mod node_config;
mod node_source_parser;

#[cfg(test)]
mod node_config_test;

pub async fn parse_node_fn(parse_spec: FnParseSpec) -> FnBuildResult<FnManifest> {
    let Runtime::Node(node_config) = &parse_spec.runtime;
    let source_parser: Arc<Box<dyn FnSourceParser>> =
        Arc::new(Box::new(NodeFnSourceParser::new(node_config.clone())));
    parse_fn_inner(&parse_spec, source_parser).await
}
