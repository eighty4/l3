use crate::runtime::node::{build_node_fn, parse_node_fn, NodeConfig};
use crate::runtime::python::{build_python_fn, parse_python_fn};
use crate::runtime::Runtime;
use crate::*;
use std::future::Future;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::process::{Command, Output};
use std::sync::Arc;

#[cfg(target_os = "windows")]
mod bin {
    pub const NODE: &str = "node.exe";
    pub const PYTHON: &str = "python.exe";
}

#[cfg(not(target_os = "windows"))]
mod bin {
    pub const NODE: &str = "node";
    pub const PYTHON: &str = "python3";
}

pub fn create_test_runtime(entrypoint: &Path) -> Arc<Box<dyn TestRuntime>> {
    let extension = entrypoint.extension().expect("source with extension");
    let runtime: Arc<Box<dyn TestRuntime>> = match extension.to_string_lossy().as_ref() {
        "js" | "mjs" | "ts" => Arc::new(Box::new(TestNodeRuntime {})),
        "py" => Arc::new(Box::new(TestPythonRuntime {})),
        _ => panic!(),
    };
    runtime
}

pub type BuildProcessResult<R> = Pin<Box<dyn Future<Output = R> + Send>>;

pub trait TestRuntime {
    fn build(&self, build_spec: FnBuildSpec) -> BuildProcessResult<FnBuildResult<FnBuildManifest>>;

    fn parse(&self, parse_spec: FnParseSpec) -> BuildProcessResult<FnParseResult<FnParseManifest>>;

    fn config(&self, project_dir: &Path) -> Runtime;

    fn verify(&self, project_dir: &Path, entrypoint: &Path) -> Option<io::Result<Output>>;
}

struct TestNodeRuntime {}

impl TestRuntime for TestNodeRuntime {
    fn build(&self, build_spec: FnBuildSpec) -> BuildProcessResult<FnBuildResult<FnBuildManifest>> {
        Box::pin(build_node_fn(build_spec))
    }

    fn parse(&self, parse_spec: FnParseSpec) -> BuildProcessResult<FnParseResult<FnParseManifest>> {
        Box::pin(parse_node_fn(parse_spec))
    }

    fn config(&self, project_dir: &Path) -> Runtime {
        Runtime::Node(Some(Arc::new(
            NodeConfig::read_configs(&project_dir).unwrap(),
        )))
    }

    fn verify(&self, project_dir: &Path, entrypoint: &Path) -> Option<io::Result<Output>> {
        Some(
            Command::new(bin::NODE)
                .arg(entrypoint)
                .current_dir(project_dir)
                .output(),
        )
    }
}

struct TestPythonRuntime {}

impl TestRuntime for TestPythonRuntime {
    fn build(&self, build_spec: FnBuildSpec) -> BuildProcessResult<FnBuildResult<FnBuildManifest>> {
        Box::pin(build_python_fn(build_spec))
    }

    fn parse(&self, parse_spec: FnParseSpec) -> BuildProcessResult<FnParseResult<FnParseManifest>> {
        Box::pin(parse_python_fn(parse_spec))
    }

    fn config(&self, _project_dir: &Path) -> Runtime {
        Runtime::Python
    }

    fn verify(&self, project_dir: &Path, entrypoint: &Path) -> Option<io::Result<Output>> {
        Some(
            Command::new(bin::PYTHON)
                .arg(entrypoint)
                .current_dir(project_dir)
                .output(),
        )
    }
}
