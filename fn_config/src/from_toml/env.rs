use std::{collections::HashMap, path::PathBuf, sync::Arc};

use l3_fn_env::parse_env_file_content;

use crate::{configs::LambdaEnvConfig, ConfigParseError, OptionallyParsable};

const TOML_DESC_ENV_FILES: &str = "array of strings";
const TOML_DESC_ENV_VARS: &str = "multiline string, array of strings or table";

impl OptionallyParsable<&toml::Table> for LambdaEnvConfig {
    fn has_value(t: &&toml::Table) -> bool {
        t.contains_key("env_vars") || t.contains_key("env_files")
    }

    fn parse(t: &toml::Table) -> Result<Self, ConfigParseError> {
        Ok(Self {
            env_files: parse_env_files_v(t.get("env_files"))?,
            env_vars: parse_env_vars_v(t.get("env_vars"))?,
        })
    }
}

fn parse_env_files_v(
    env_vars_v: Option<&toml::Value>,
) -> Result<Option<Vec<Arc<PathBuf>>>, ConfigParseError> {
    match env_vars_v {
        Some(toml::Value::Array(env_files_a)) => {
            let mut env_files = Vec::new();
            for env_file_v in env_files_a {
                match env_file_v {
                    toml::Value::String(env_file_s) => {
                        env_files.push(Arc::new(PathBuf::from(env_file_s)))
                    }
                    _ => {
                        return Err(ConfigParseError::Misconfigured {
                            actual: env_file_v.type_str().into(),
                            expected: TOML_DESC_ENV_FILES.into(),
                            field: "env_files".into(),
                        })
                    }
                }
            }
            Ok(Some(env_files))
        }
        Some(unsupported) => Err(ConfigParseError::Misconfigured {
            actual: unsupported.type_str().into(),
            expected: TOML_DESC_ENV_FILES.into(),
            field: "env_files".into(),
        }),
        _ => Ok(None),
    }
}

fn parse_env_vars_v(
    env_vars_v: Option<&toml::Value>,
) -> Result<Option<HashMap<String, String>>, ConfigParseError> {
    match env_vars_v {
        Some(toml::Value::Table(t)) => parse_env_vars_t(t),
        Some(toml::Value::String(s)) => parse_env_vars_s(s),
        Some(toml::Value::Array(a)) => parse_env_vars_a(a),
        Some(unsupported) => Err(ConfigParseError::Misconfigured {
            actual: unsupported.type_str().into(),
            expected: TOML_DESC_ENV_VARS.into(),
            field: "env_vars".into(),
        }),
        _ => Ok(None),
    }
}

fn parse_env_vars_s(env_vars_s: &str) -> Result<Option<HashMap<String, String>>, ConfigParseError> {
    Ok(Some(parse_env_file_content(env_vars_s)?))
}

fn parse_env_vars_t(
    env_vars_t: &toml::Table,
) -> Result<Option<HashMap<String, String>>, ConfigParseError> {
    let mut env_vars = HashMap::new();
    for (env_var_k, env_var_v) in env_vars_t {
        env_vars.insert(
            env_var_k.to_string(),
            match env_var_v {
                toml::Value::String(v) => v.to_string(),
                toml::Value::Integer(v) => v.to_string(),
                toml::Value::Float(v) => v.to_string(),
                toml::Value::Boolean(v) => v.to_string(),
                toml::Value::Datetime(v) => v.to_string(),
                _ => {
                    return Err(ConfigParseError::Misconfigured {
                        actual: format!("table with {}", env_var_v.type_str()),
                        expected: TOML_DESC_ENV_VARS.into(),
                        field: "env_vars".into(),
                    })
                }
            },
        );
    }
    Ok(Some(env_vars))
}

fn parse_env_vars_a(
    env_vars_a: &Vec<toml::Value>,
) -> Result<Option<HashMap<String, String>>, ConfigParseError> {
    let mut env_vars = HashMap::new();
    for env_var_v in env_vars_a {
        match env_var_v.as_str() {
            Some(env_var_s) => {
                match env_var_s.split_once("=") {
                    Some((k, v)) => env_vars.insert(k.to_string(), v.to_string()),
                    None => env_vars.insert(env_var_s.to_string(), "".to_string()),
                };
            }
            None => {
                return Err(ConfigParseError::Misconfigured {
                    actual: format!("array with {}", env_var_v.type_str()),
                    expected: TOML_DESC_ENV_VARS.into(),
                    field: "env_vars".into(),
                })
            }
        }
    }
    Ok(Some(env_vars))
}
