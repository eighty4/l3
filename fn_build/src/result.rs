use crate::FnBuildOutput;
use serde::Deserialize;
use std::io;
use std::path::PathBuf;

#[derive(Clone, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
#[cfg_attr(test, serde(rename_all = "camelCase"))]
pub enum ModuleImport {
    #[allow(unused)]
    PackageDependency {
        package: String,
        subpath: Option<String>,
    },
    /// Relative path to a source file from the project root.
    RelativeSource(PathBuf),
    #[allow(unused)]
    Unknown(String),
}

#[derive(Clone, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct FnSource {
    pub imports: Vec<ModuleImport>,
    pub path: PathBuf,
}

#[derive(Clone, Deserialize)]
#[cfg_attr(test, serde(rename_all = "camelCase"))]
pub enum FnDependencies {
    Required,
    Unused,
}

#[derive(Clone, Deserialize)]
pub struct FnManifest {
    pub dependencies: FnDependencies,
    pub sources: Vec<FnSource>,
}

#[derive(Clone, Deserialize)]
pub struct FnBuild {
    #[allow(unused)]
    pub manifest: FnManifest,
    #[allow(unused)]
    pub output: FnBuildOutput,
}

#[derive(thiserror::Error, Debug)]
pub enum FnBuildError {
    #[error("entrypoint file type is unsupported")]
    InvalidFileType,
    #[error("{0}")]
    IoError(#[from] io::Error),
}

pub type FnBuildResult<T> = Result<T, FnBuildError>;

impl From<PathBuf> for FnSource {
    fn from(path: PathBuf) -> Self {
        Self {
            imports: Vec::new(),
            path,
        }
    }
}
