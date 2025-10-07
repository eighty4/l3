use crate::{Language, NodeVersion, PythonVersion};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

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
    pub language: Option<Language>,
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PythonLambdaConfig {
    pub version: Option<PythonVersion>,
}

#[derive(Debug, PartialEq)]
pub struct LambdaEnvConfig {
    pub env_files: Option<Vec<Arc<PathBuf>>>,
    pub env_vars: Option<HashMap<String, String>>,
}
