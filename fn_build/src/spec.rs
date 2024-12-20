use crate::runtime::Runtime;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum BuildMode {
    Debug,
    Release,
}

pub struct FnParseSpec {
    pub entrypoint: PathBuf,
    pub project_dir: Arc<PathBuf>,
    pub runtime: Runtime,
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
