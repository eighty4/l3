use crate::runtime::node::NodeConfigError;
use crate::runtime::Runtime;
use l3_fn_config::Language;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

pub struct FnParseSpec {
    pub entrypoint: PathBuf,
    pub project_dir: Arc<PathBuf>,
    pub runtime: Runtime,
}

impl FnParseSpec {
    pub fn language(&self) -> FnParseResult<Language> {
        Language::try_from(self.entrypoint.as_path()).map_err(|_| FnParseError::InvalidFileType)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FnDependencies {
    Required,
    Unused,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FnEntrypoint {
    /// Exported functions from entrypoint source file.
    pub handlers: Vec<String>,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ModuleImport {
    PackageDependency {
        package: String,
        subpath: Option<String>,
    },
    /// A source file import specified by relative path from the project's root.
    RelativeSource(PathBuf),
    /// An unresolvable import specifier.
    Unknown(String),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FnSource {
    pub imports: Vec<ModuleImport>,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FnParseManifest {
    pub dependencies: FnDependencies,
    pub entrypoint: FnEntrypoint,
    pub sources: Vec<FnSource>,
}

impl FnParseManifest {
    pub fn source_paths(&self) -> Vec<PathBuf> {
        self.sources.iter().map(|s| s.path.clone()).collect()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FnParseError {
    #[error("entrypoint file type is unsupported")]
    InvalidFileType,
    #[error("{0}")]
    IoError(#[from] io::Error),
    #[error("could not find entrypoint {0}")]
    MissingEntrypoint(PathBuf),
    #[error("entrypoint {0} does not have a handler fn {1}")]
    MissingHandler(PathBuf, String),
    #[error("error reading package.json: {0}")]
    NodeConfig(#[from] NodeConfigError),
    #[error("syntax error")]
    SyntaxError,
    #[error("could not resolve \"{import}\" from \"{from}\"")]
    UnresolvedImport { from: PathBuf, import: String },
}

pub type FnParseResult<T> = Result<T, FnParseError>;
