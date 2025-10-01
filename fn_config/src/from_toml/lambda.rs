use std::path::PathBuf;

use crate::{
    configs::LambdaConfig, configs::LambdaEnvConfig, configs::LambdaRuntimeConfig,
    ConfigParseError, OptionallyParsable,
};

impl TryFrom<&toml::Table> for LambdaConfig {
    type Error = ConfigParseError;

    fn try_from(lambda_t: &toml::Table) -> Result<Self, Self::Error> {
        Ok(LambdaConfig {
            name: lambda_t
                .get("name")
                .and_then(|name_v| name_v.as_str().map(String::from)),
            source: lambda_t
                .get("source")
                .and_then(|source_v| source_v.as_str().map(PathBuf::from)),
            handler: lambda_t
                .get("handler")
                .and_then(|handler_v| handler_v.as_str().map(String::from)),
            env: LambdaEnvConfig::optionally_parse(lambda_t)?,
            runtime: LambdaRuntimeConfig::optionally_parse(lambda_t)?,
        })
    }
}
