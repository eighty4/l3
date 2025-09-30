use crate::checksum::Checksum;
use crate::runtime::Runtime;
use crate::{FnDependencies, FnHandler, FnParseError, FnParseSpec, FnSource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum BuildMode {
    Debug,
    Release,
}

/// Output config for a function build. A function's output will be written to
/// a path constructed from: `build_root/build_mode?/dirname`.
///
/// FnOutputConfig's dirname must be unique within a batch build process.
#[derive(Clone, Deserialize)]
pub struct FnOutputConfig {
    /// Absolute path to root build directory. This path will either contain the build_mode
    /// partitioned outputs or directly contain the function builds depending on
    /// the value of use_build_mode.
    pub build_root: PathBuf,
    /// Whether to create a {lambda_fn_name}.zip archive file in the output directory.
    pub create_archive: bool,
    /// Output directory of the function within the build directory.
    pub dirname: String,
    /// Build output will be nested in build_root by the BuildMode of the build, such as
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FnBuildOutput {
    /// Path to .zip file of build output if configured by FnOutputConfig.
    pub archive_file: Option<PathBuf>,
    /// PathBuf to the build's output directory as resolved by FnOutputConfig.
    pub build_dir: PathBuf,
    // Map of source paths to their original or rewritten name as they.
    pub paths: HashMap<PathBuf, PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
