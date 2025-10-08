use std::{
    fmt,
    path::{Path, PathBuf},
};

use anyhow::anyhow;

use crate::ConfigParseError;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum UnresolvedLanguage {
    #[error("{0} is missing a file extension")]
    NoExtension(String),
    #[error("{0} does not have a supported file extension")]
    UnsupportedExtension(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Language {
    JavaScript,
    Python,
    TypeScript,
}

impl TryFrom<&Path> for Language {
    type Error = UnresolvedLanguage;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => match ext {
                "js" | "mjs" => Ok(Language::JavaScript),
                "py" => Ok(Language::Python),
                "ts" => Ok(Language::TypeScript),
                _ => Err(UnresolvedLanguage::UnsupportedExtension(
                    path.extension().unwrap().to_string_lossy().to_string(),
                )),
            },
            _ => Err(UnresolvedLanguage::NoExtension(
                path.file_name().unwrap().to_string_lossy().to_string(),
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LambdaRouting {
    HttpRoute(HttpRoute),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
// #[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Delete,
    Patch,
    Post,
    Put,
}

impl<'a> TryFrom<&'a str> for HttpMethod {
    type Error = anyhow::Error;

    fn try_from(http_method_str: &'a str) -> Result<Self, Self::Error> {
        let http_method = match http_method_str.to_uppercase().as_str() {
            "DELETE" => Some(HttpMethod::Delete),
            "GET" => Some(HttpMethod::Get),
            "PATCH" => Some(HttpMethod::Patch),
            "POST" => Some(HttpMethod::Post),
            "PUT" => Some(HttpMethod::Put),
            _ => None,
        };
        http_method.ok_or(anyhow!(
            "could not resolve http method from {http_method_str}"
        ))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HttpRoute {
    pub method: HttpMethod,
    pub path: String,
}

impl HttpRoute {
    pub fn new(path: &Path, handler_fn_name: &str) -> Result<Self, anyhow::Error> {
        debug_assert!(path.is_relative());
        match Self::extract_http_path(path) {
            None => Err(anyhow!("not an http route source path")),
            Some(path) => match HttpMethod::try_from(handler_fn_name) {
                Err(err) => Err(err),
                Ok(method) => Ok(Self { method, path }),
            },
        }
    }

    fn extract_http_path(path: &Path) -> Option<String> {
        debug_assert!(path.is_relative());
        let mut parts: Vec<String> = Vec::new();
        for p in path.parent()?.components().rev() {
            if p.as_os_str().to_string_lossy().as_ref() == "routes" {
                return Some(parts.join("/"));
            } else {
                parts.insert(0, p.as_os_str().to_string_lossy().to_string());
            }
        }
        None
    }
}

#[derive(Debug, PartialEq)]
pub struct LambdaSpec {
    pub name: String,
    pub source: PathBuf,
    pub handler: String,
    pub runtime: LambdaRuntimeSpec,
}

#[derive(Debug, PartialEq)]
pub enum LambdaRuntimeSpec {
    Node,
    Python,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NodeVersion {
    TwentyFour,
    TwentyTwo,
    Twenty,
    Eighteen,
}

impl fmt::Display for NodeVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NodeVersion::TwentyFour => "24",
                NodeVersion::TwentyTwo => "22",
                NodeVersion::Twenty => "20",
                NodeVersion::Eighteen => "18",
            }
        )
    }
}

impl TryFrom<&str> for NodeVersion {
    type Error = ConfigParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "24" => Ok(NodeVersion::TwentyFour),
            "22" => Ok(NodeVersion::TwentyTwo),
            "20" => Ok(NodeVersion::Twenty),
            "18" => Ok(NodeVersion::Eighteen),
            _ => Err(ConfigParseError::Misconfigured {
                field: "node version".into(),
                actual: s.into(),
                expected: "one of (24|22|20|18)".into(),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum PythonVersion {
    ThreeFourteen,
    ThreeThirteen,
    ThreeTwelve,
    ThreeTen,
    ThreeNine,
}

impl fmt::Display for PythonVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PythonVersion::ThreeFourteen => "3.14",
                PythonVersion::ThreeThirteen => "3.13",
                PythonVersion::ThreeTwelve => "3.12",
                PythonVersion::ThreeTen => "3.10",
                PythonVersion::ThreeNine => "3.9",
            }
        )
    }
}

impl TryFrom<&str> for PythonVersion {
    type Error = ConfigParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "3.14" => Ok(PythonVersion::ThreeFourteen),
            "3.13" => Ok(PythonVersion::ThreeThirteen),
            "3.12" => Ok(PythonVersion::ThreeTwelve),
            "3.10" => Ok(PythonVersion::ThreeTen),
            "3.9" => Ok(PythonVersion::ThreeNine),
            _ => Err(ConfigParseError::Misconfigured {
                field: "python version".into(),
                actual: s.into(),
                expected: "one of (3.14|3.13|3.12|3.10|3.9)".into(),
            }),
        }
    }
}
