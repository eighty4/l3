use crate::{
    configs::LambdaRuntimeConfig, configs::NodeLambdaConfig, configs::NodeVersion,
    configs::PythonLambdaConfig, configs::PythonVersion, ConfigParseError, OptionallyParsable,
};

impl OptionallyParsable<&toml::Table> for LambdaRuntimeConfig {
    fn has_value(l3_t: &&toml::Table) -> bool {
        l3_t.contains_key("node_opts")
            || l3_t.contains_key("node_version")
            || l3_t.contains_key("python_opts")
            || l3_t.contains_key("python_version")
    }

    fn parse(t: &toml::Table) -> Result<Self, ConfigParseError> {
        Ok(Self {
            node: parse_node_config(t)?,
            python: parse_python_config(t)?,
        })
    }
}

fn parse_node_config(t: &toml::Table) -> Result<Option<NodeLambdaConfig>, ConfigParseError> {
    let version = parse_node_version(t.get("node_version"))?;
    match t.get("node_opts") {
        Some(toml::Value::Table(node_opts_t)) => Ok(Some(NodeLambdaConfig {
            version: match parse_node_version(node_opts_t.get("version"))? {
                Some(node_opts_version) => {
                    if version
                        .as_ref()
                        .map(|v| v == &node_opts_version)
                        .unwrap_or(true)
                    {
                        Some(node_opts_version)
                    } else {
                        return Err(ConfigParseError::Misconfigured {
                            actual: "node_version and node_opts.version do not match".into(),
                            expected: "use node_version or node_opts.version".into(),
                            field: "node_opts.version".into(),
                        });
                    }
                }
                None => version,
            },
        })),
        None => {
            if version.is_none() {
                Ok(None)
            } else {
                Ok(Some(NodeLambdaConfig { version }))
            }
        }
        _ => panic!(),
    }
}

fn parse_node_version(
    node_version_v: Option<&toml::Value>,
) -> Result<Option<NodeVersion>, ConfigParseError> {
    match node_version_v {
        Some(toml::Value::String(version_s)) => {
            Ok(Some(NodeVersion::try_from(version_s.as_str())?))
        }
        Some(toml::Value::Integer(version_i)) => {
            Ok(Some(NodeVersion::try_from(version_i.to_string().as_str())?))
        }
        None => Ok(None),
        _ => panic!(),
    }
}

fn parse_python_config(t: &toml::Table) -> Result<Option<PythonLambdaConfig>, ConfigParseError> {
    let version = parse_python_version(t.get("python_version"))?;
    match t.get("python_opts") {
        Some(toml::Value::Table(python_opts_t)) => Ok(Some(PythonLambdaConfig {
            version: match parse_python_version(python_opts_t.get("version"))? {
                Some(python_opts_version) => {
                    if version
                        .as_ref()
                        .map(|v| v == &python_opts_version)
                        .unwrap_or(true)
                    {
                        Some(python_opts_version)
                    } else {
                        return Err(ConfigParseError::Misconfigured {
                            actual: "python_version and python_opts.version do not match".into(),
                            expected: "use python_version or python_opts.version".into(),
                            field: "python_opts.version".into(),
                        });
                    }
                }
                None => version,
            },
        })),
        None => {
            if version.is_none() {
                Ok(None)
            } else {
                Ok(Some(PythonLambdaConfig { version }))
            }
        }
        _ => panic!(),
    }
}

fn parse_python_version(
    node_version_v: Option<&toml::Value>,
) -> Result<Option<PythonVersion>, ConfigParseError> {
    match node_version_v {
        Some(toml::Value::String(version_s)) => {
            Ok(Some(PythonVersion::try_from(version_s.as_str())?))
        }
        Some(toml::Value::Float(version_f)) => Ok(Some(PythonVersion::try_from(
            version_f.to_string().as_str(),
        )?)),
        None => Ok(None),
        _ => panic!(),
    }
}
