use crate::checksum::Checksum;
use crate::runtime::Runtime;
use crate::{FnDependencies, FnHandler, FnParseError, FnParseSpec, FnSource};
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum BuildMode {
    Debug,
    Release,
}

/// Output config for the build.
#[derive(Clone, Deserialize)]
pub struct FnOutputConfig {
    /// Absolute path to root build directory, parent of debug and release output directories if
    /// use_build_mode is true. A function's build output will be a directory with the name of the
    /// function's unique identifier.
    pub build_root: PathBuf,
    /// Whether to create a {fn_identifier}.zip archive file in the output directory.
    pub create_archive: bool,
    /// Build output will be nested in build_root by the build's BuildMode, such as
    /// `/example/build/debug` and `/example/build/release`.
    pub use_build_mode: bool,
}

pub struct FnBuildSpec {
    pub entrypoint: PathBuf,
    pub handler_fn_name: String,
    pub mode: BuildMode,
    pub output: FnOutputConfig,
    pub project_dir: Arc<PathBuf>,
    pub runtime: Runtime,
}

impl FnBuildSpec {
    pub fn to_parse_spec(&self) -> FnParseSpec {
        FnParseSpec {
            entrypoint: self.entrypoint.clone(),
            project_dir: self.project_dir.clone(),
            runtime: self.runtime.clone(),
        }
    }

    pub fn output_build_root(&self) -> PathBuf {
        if self.output.use_build_mode {
            self.output.build_root.join(match self.mode {
                BuildMode::Debug => "debug",
                BuildMode::Release => "release",
            })
        } else {
            self.output.build_root.clone()
        }
    }

    pub fn output_archive_file(&self) -> Option<PathBuf> {
        if self.output.create_archive {
            Some(self.output.build_root.join(match self.mode {
                BuildMode::Debug => "debug.zip",
                BuildMode::Release => "release.zip",
            }))
        } else {
            None
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct FnBuildOutput {
    pub archive_file: Option<PathBuf>,
    /// PathBuf to the build's output directory partitioned by the build's BuildMode.
    pub build_dir: PathBuf,
}

#[derive(Clone, Deserialize)]
pub struct FnBuildManifest {
    /// Checksums of the original sources of the function's build.
    pub checksums: HashMap<PathBuf, Checksum>,
    pub dependencies: FnDependencies,
    pub entrypoint: PathBuf,
    pub handler: FnHandler,
    pub output: FnBuildOutput,
    pub sources: Vec<FnSource>,
}

#[derive(thiserror::Error, Debug)]
pub enum FnBuildError {
    #[error("{0}")]
    IoError(#[from] io::Error),
    #[error("error parsing function: {0}")]
    ParseError(#[from] FnParseError),
    #[error("build task error: {0}")]
    KnownError(#[from] anyhow::Error),
}

pub type FnBuildResult<T> = Result<T, FnBuildError>;
