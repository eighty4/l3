use crate::runtime::Runtime;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum BuildMode {
    Debug,
    Release,
}

/// The output method for a function build. Builds written to disk will be keyed by a build's
/// BuildMode. The build output for FnBuildOutput::Directory or FnBuildOutput::Archive's build_root
/// with a value of /example/build and a build mode of BuildMode::Debug will be built at
/// /example/build/debug.
#[derive(Clone, Deserialize)]
pub enum FnBuildOutput {
    /// FnBuildOutput::Archive's archive_file must be a non-existing absolute path to the generated
    /// archive.
    Archive {
        archive_file: PathBuf,
        build_root: PathBuf,
    },
    Directory(PathBuf),
}

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
