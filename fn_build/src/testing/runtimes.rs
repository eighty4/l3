use crate::runtime::node::{build_node_fn, parse_node_fn, NodeConfig};
use crate::runtime::Runtime;
use crate::*;
use std::future::Future;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::process::{Command, Output};
use std::sync::Arc;

pub type BuildProcessResult<T> = Pin<Box<dyn Future<Output = Result<T, FnBuildError>> + Send>>;

pub trait TestRuntime {
    fn build(&self, build_spec: FnBuildSpec) -> BuildProcessResult<FnBuild>;

    fn parse(&self, parse_spec: FnParseSpec) -> BuildProcessResult<FnManifest>;

    fn config(&self, project_dir: &Path) -> Runtime;

    fn verify(&self, project_dir: &Path, entrypoint: &Path) -> Option<io::Result<Output>>;
}

pub struct TestNodeRuntime {}

impl TestRuntime for TestNodeRuntime {
    fn build(&self, build_spec: FnBuildSpec) -> BuildProcessResult<FnBuild> {
        Box::pin(build_node_fn(build_spec))
    }

    fn parse(&self, parse_spec: FnParseSpec) -> BuildProcessResult<FnManifest> {
        Box::pin(parse_node_fn(parse_spec))
    }

    fn config(&self, project_dir: &Path) -> Runtime {
        Runtime::Node(Arc::new(
            NodeConfig::read_node_config(&project_dir).unwrap(),
        ))
    }

    fn verify(&self, project_dir: &Path, entrypoint: &Path) -> Option<io::Result<Output>> {
        Some(
            Command::new("node")
                .arg(entrypoint)
                .current_dir(project_dir)
                .output(),
        )
    }
}
