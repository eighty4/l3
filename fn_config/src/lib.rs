mod configs;
mod from_toml;

#[cfg(test)]
mod lib_test;

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::Arc,
};

use l3_fn_env::EnvVarsParseError;

use configs::{LambdaRuntimeConfig, ProjectConfig};

use crate::configs::{NodeLambdaConfig, PythonLambdaConfig};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ConfigParseError {
    #[error("{0}")]
    EnvVarsParseError(#[from] EnvVarsParseError),
    #[error("io error reading {path} env file: {kind}")]
    IoError {
        path: PathBuf,
        kind: std::io::ErrorKind,
    },
    #[error("{0}")]
    TomlParseError(#[from] toml::de::Error),
    #[error("{field} is {actual} and must be {expected}")]
    Misconfigured {
        field: String,
        expected: String,
        actual: String,
    },
    // #[error("error parsing {file_name} env vars: {cause}")]
    // SyntaxError {
    //     cause: String,
    //     content: String,
    //     file_name: String,
    //     line: usize,
    // },
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
    Node(NodeLambdaConfig),
    Python(PythonLambdaConfig),
}

#[derive(Default)]
pub struct ConfigUpdate {
    pub config_errs: Vec<ConfigParseError>,
    pub mutations: HashSet<UpdateMutation>,
}

impl ConfigUpdate {
    fn extend(&mut self, other: ConfigUpdate) {
        self.config_errs.extend(other.config_errs);
        self.mutations.extend(other.mutations);
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum UpdateMutation {
    ProjectEnv,
}

pub struct LLLConfigs {
    lambdas: HashMap<String, Arc<LambdaSpec>>,
    project: Option<ProjectConfig>,
    project_dir: Arc<PathBuf>,
}

impl LLLConfigs {
    pub fn new(project_dir: Arc<PathBuf>) -> Self {
        Self {
            project_dir,
            lambdas: HashMap::new(),
            project: None,
        }
    }

    // lookup any discrete or routing configured lambda by name
    pub fn lambda_by_name(&self, name: &str) -> Option<Arc<LambdaSpec>> {
        self.lambdas.get(name).cloned()
    }

    // lambdas configured via l3.toml [[lambdas]]
    pub fn discrete_lambdas(&self) -> Vec<Arc<LambdaSpec>> {
        self.lambdas.values().cloned().collect()
    }

    // env file paths configured via l3.toml [env_files]
    // fn project_env_files(&self) -> Option<&Vec<Arc<PathBuf>>> {
    //     self.project
    //         .as_ref()
    //         .and_then(|p| p.env.as_ref())
    //         .and_then(|e| e.env_files.as_ref())
    // }

    // env vars configured via l3.toml [env]
    // fn project_env_vars(&self) -> Option<&HashMap<String, String>> {
    //     self.project
    //         .as_ref()
    //         .and_then(|p| p.env.as_ref())
    //         .and_then(|e| e.env_vars.as_ref())
    // }

    pub fn update_all_configs(&mut self) -> ConfigUpdate {
        let paths = ["l3.toml"].iter().map(PathBuf::from).collect();
        self.update_configs(&paths)
    }

    pub fn update_configs(&mut self, paths: &Vec<PathBuf>) -> ConfigUpdate {
        let mut result = ConfigUpdate::default();
        for path in paths {
            match path.to_str() {
                Some("l3.toml") => {
                    result.extend(self.update_project_config());
                }
                _ => panic!(),
            }
        }
        result
    }

    fn update_project_config(&mut self) -> ConfigUpdate {
        let mut result = ConfigUpdate::default();
        let pc = match self.read_from_project_dir(&"l3.toml".into()) {
            Ok(pc) => ProjectConfig::try_from(&pc.parse::<toml::Table>().unwrap()).unwrap(),
            Err(err) => {
                result.config_errs.push(err);
                return result;
            }
        };

        for lambda in &pc.lambdas {
            if let (Some(name), Some(handler), Some(source)) = (
                lambda.name.clone(),
                lambda.handler.clone(),
                lambda.source.clone(),
            ) {
                let runtime = if source
                    .as_path()
                    .extension()
                    .map(|ext| ext == "ts" || ext == "js")
                    .unwrap()
                {
                    LambdaRuntimeSpec::Node(match &pc.runtime {
                        Some(LambdaRuntimeConfig {
                            node: Some(nrc), ..
                        }) => nrc.clone(),
                        _ => NodeLambdaConfig::default(),
                    })
                } else {
                    LambdaRuntimeSpec::Python(match &pc.runtime {
                        Some(LambdaRuntimeConfig {
                            python: Some(prc), ..
                        }) => prc.clone(),
                        _ => PythonLambdaConfig::default(),
                    })
                };
                self.lambdas.insert(
                    lambda.name.clone().unwrap(),
                    Arc::new(LambdaSpec {
                        name,
                        handler,
                        source,
                        runtime,
                    }),
                );
            } else {
                panic!();
            }
        }

        self.project = Some(pc);
        result
    }

    // read path relative to project dir
    fn read_from_project_dir(&self, p: &PathBuf) -> Result<String, ConfigParseError> {
        debug_assert!(p.is_relative());
        fs::read_to_string(self.project_dir.join(p)).map_err(|err| ConfigParseError::IoError {
            path: p.clone(),
            kind: err.kind(),
        })
    }
}

pub trait OptionallyParsable<T>: Sized {
    fn has_value(value: &T) -> bool;

    fn optionally_parse(value: T) -> Result<Option<Self>, ConfigParseError> {
        if Self::has_value(&value) {
            Self::parse(value).map(Some)
        } else {
            Ok(None)
        }
    }

    fn parse(value: T) -> Result<Self, ConfigParseError>;
}
