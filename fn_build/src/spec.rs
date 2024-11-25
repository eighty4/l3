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
///
/// Builds will be keyed in FnBuildOutput::build_dir by the build's BuildMode, such as
/// `/example/build/debug` and `/example/build/release`.
#[derive(Clone, Deserialize)]
pub struct FnOutputConfig {
    /// Absolute path to root build directory, parent of debug and release output directories.
    pub build_root: PathBuf,
    /// Providing an archive filename will create an archive of the build output in build_root.
    /// With a build_dir of /example/build, BuildMode::Debug and create_archive of true,
    /// an archive of the build output will be created at /example/build/debug.zip.
    pub create_archive: bool,
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
        self.output.build_root.join(match self.mode {
            BuildMode::Debug => "debug",
            BuildMode::Release => "release",
        })
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
