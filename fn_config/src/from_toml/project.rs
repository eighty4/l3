use crate::{
    configs::LambdaConfig, configs::LambdaEnvConfig, configs::LambdaRuntimeConfig,
    configs::ProjectConfig, ConfigParseError, OptionallyParsable,
};

impl TryFrom<&toml::Table> for ProjectConfig {
    type Error = ConfigParseError;

    fn try_from(project_t: &toml::Table) -> Result<Self, Self::Error> {
        let lambdas = match project_t.get("lambda") {
            None => Vec::new(),
            Some(lambda_v) => match lambda_v {
                toml::Value::Array(lambda_a) => {
                    let mut lambdas = Vec::new();
                    for lambda_v in lambda_a {
                        match lambda_v {
                            toml::Value::Table(lambda_t) => {
                                lambdas.push(LambdaConfig::try_from(lambda_t)?)
                            }
                            _ => panic!(),
                        }
                    }
                    lambdas
                }
                toml::Value::Table(lambda_t) => vec![LambdaConfig::try_from(lambda_t)?],
                _ => panic!("toml key lambda is neither [lambda] nor [[lambda]]"),
            },
        };
        Ok(Self {
            project_name: project_t
                .get("name")
                .and_then(|name_v| name_v.as_str().map(|s| s.to_string())),
            lambda_naming: project_t
                .get("naming")
                .and_then(|naming_v| naming_v.as_str().map(|s| s.to_string())),
            lambdas,
            env: LambdaEnvConfig::optionally_parse(project_t)?,
            runtime: LambdaRuntimeConfig::optionally_parse(project_t)?,
        })
    }
}
