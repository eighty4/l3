use crate::runtime::Runtime;
use crate::{FnRouting, HttpMethod};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct FnParseSpec {
    pub entrypoint: PathBuf,
    pub project_dir: Arc<PathBuf>,
    pub runtime: Runtime,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FnDependencies {
    Required,
    Unused,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FnHandler {
    /// Function name matching case in source file.
    pub fn_name: String,
    /// Routing method as the function would be deployed.
    pub routing: FnRouting,
}

impl FnHandler {
    pub fn from_handler_fn(path: &Path, fn_name: String) -> Self {
        let routing = FnRouting::from_handler_fn(path, fn_name.as_str());
        Self { fn_name, routing }
    }

    pub fn to_fn_identifier(&self) -> String {
        match &self.routing {
            FnRouting::Unsupported => panic!(),
            FnRouting::HttpRoute(http_route) => {
                format!(
                    "{}-{}",
                    http_route.path.replace("/", "-"),
                    match http_route.method {
                        HttpMethod::Get => "get",
                        HttpMethod::Delete => "delete",
                        HttpMethod::Patch => "patch",
                        HttpMethod::Post => "post",
                        HttpMethod::Put => "put",
                    }
                )
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FnEntrypoint {
    pub handlers: Vec<FnHandler>,
    pub path: PathBuf,
}

impl FnEntrypoint {
    pub fn to_fn_identifier(&self, handler_fn_name: &str) -> FnParseResult<String> {
        self.handlers
            .iter()
            .find(|h| h.fn_name == handler_fn_name)
            .map(FnHandler::to_fn_identifier)
            .ok_or_else(|| {
                FnParseError::MissingHandler(self.path.clone(), handler_fn_name.to_string())
            })
    }
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

#[derive(Clone, Deserialize, Serialize)]
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
    #[error("entrypoint {0} does not have a handler fn {1}")]
    MissingHandler(PathBuf, String),
    #[error("syntax error")]
    SyntaxError,
}

pub type FnParseResult<T> = Result<T, FnParseError>;
