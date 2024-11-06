use crate::spec::FnBuildOutput;
use serde::Deserialize;
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
    /// Absolute path to a project directory source file
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

pub type FnSources = Vec<FnSource>;

pub struct FnBuild {
    #[allow(unused)]
    pub output: FnBuildOutput,
    #[allow(unused)]
    pub sources: FnSources,
}

#[derive(thiserror::Error, Debug)]
pub enum FnBuildError {
    #[error("entrypoint file type is unsupported")]
    InvalidFileType,
}

pub type FnBuildResult<T> = Result<T, FnBuildError>;
