use crate::runtime::Runtime;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum BuildMode {
    Debug,
    Release,
}

pub type FnBuildOutput = PathBuf;

// pub enum FnBuildOutput {
//     Archive(PathBuf),
//     Directory(PathBuf),
// }

pub struct FnBuildSpec {
    pub function: FnParseSpec,
    pub mode: BuildMode,
    pub output: FnBuildOutput,
}

#[derive(Clone)]
pub struct FnParseSpec {
    pub entrypoint: PathBuf,
    pub project_dir: Arc<PathBuf>,
    pub runtime: Runtime,
}
