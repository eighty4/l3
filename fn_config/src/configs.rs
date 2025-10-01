use crate::ConfigParseError;
use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};

#[derive(Debug, Default, PartialEq)]
pub struct ProjectConfig {
    pub project_name: Option<String>,
    pub lambda_naming: Option<String>,
    pub lambdas: Vec<LambdaConfig>,
    pub env: Option<LambdaEnvConfig>,
    pub runtime: Option<LambdaRuntimeConfig>,
}

#[derive(Debug, Default, PartialEq)]
pub struct LambdaConfig {
    pub name: Option<String>,
    pub source: Option<PathBuf>,
    pub handler: Option<String>,
    pub env: Option<LambdaEnvConfig>,
    pub runtime: Option<LambdaRuntimeConfig>,
}

#[derive(Debug, PartialEq)]
pub struct LambdaRuntimeConfig {
    pub node: Option<NodeLambdaConfig>,
    pub python: Option<PythonLambdaConfig>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NodeLambdaConfig {
    pub version: Option<NodeVersion>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeVersion {
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
            "22" => Ok(NodeVersion::TwentyTwo),
            "20" => Ok(NodeVersion::Twenty),
            "18" => Ok(NodeVersion::Eighteen),
            _ => panic!(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PythonLambdaConfig {
    pub version: Option<PythonVersion>,
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
            _ => panic!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LambdaEnvConfig {
    pub env_files: Option<Vec<Arc<PathBuf>>>,
    pub env_vars: Option<HashMap<String, String>>,
}
