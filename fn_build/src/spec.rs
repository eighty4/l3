use crate::runtime::Runtime;
use std::path::PathBuf;

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

pub struct FnParseSpec {
    pub entrypoint: PathBuf,
    pub project_dir: PathBuf,
    pub runtime: Runtime,
}
