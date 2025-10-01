use std::collections::HashMap;

use crate::{configs::LambdaEnvConfig, OptionallyParsable};

#[test]
fn test_parse_env_toml_with_env_vars_table_of_strings_and_numbers() {
    let s = r#"
[env_vars]
API_CLIENT_ID=123
API_CLIENT_SECRET="im a secret"
API_ENDPOINT=""
API_ROLLOUT=true
API_ROLLOUT_ROUTING=0.1
"#;
    assert_eq!(
        LambdaEnvConfig::parse(&s.parse::<toml::Table>().unwrap()),
        Ok(LambdaEnvConfig {
            env_files: None,
            env_vars: Some(HashMap::from([
                ("API_CLIENT_ID".into(), "123".into()),
                ("API_CLIENT_SECRET".into(), "im a secret".into()),
                ("API_ENDPOINT".into(), "".into()),
                ("API_ROLLOUT".into(), "true".into()),
                ("API_ROLLOUT_ROUTING".into(), "0.1".into()),
            ]))
        })
    );
}

#[test]
fn test_parse_env_toml_with_env_vars_multiline_string() {
    let s = r#"
env_vars = """
API_CLIENT_ID=123
API_CLIENT_SECRET="im a secret"
API_ENDPOINT=
API_ROLLOUT=true
API_ROLLOUT_ROUTING=0.1
"""
"#;
    assert_eq!(
        LambdaEnvConfig::parse(&s.parse::<toml::Table>().unwrap()),
        Ok(LambdaEnvConfig {
            env_files: None,
            env_vars: Some(HashMap::from([
                ("API_CLIENT_ID".into(), "123".into()),
                ("API_CLIENT_SECRET".into(), "im a secret".into()),
                ("API_ENDPOINT".into(), "".into()),
                ("API_ROLLOUT".into(), "true".into()),
                ("API_ROLLOUT_ROUTING".into(), "0.1".into()),
            ]))
        })
    );
}

#[test]
fn test_parse_env_toml_with_env_vars_array_of_string() {
    let s = r#"
env_vars = [
  "API_CLIENT_ID=123",
  "API_CLIENT_SECRET=im a secret",
  "API_ENDPOINT=",
  "API_ROLLOUT=true",
  "API_ROLLOUT_ROUTING=0.1"
]
"#;
    assert_eq!(
        LambdaEnvConfig::parse(&s.parse::<toml::Table>().unwrap()),
        Ok(LambdaEnvConfig {
            env_files: None,
            env_vars: Some(HashMap::from([
                ("API_CLIENT_ID".into(), "123".into()),
                ("API_CLIENT_SECRET".into(), "im a secret".into()),
                ("API_ENDPOINT".into(), "".into()),
                ("API_ROLLOUT".into(), "true".into()),
                ("API_ROLLOUT_ROUTING".into(), "0.1".into()),
            ]))
        })
    );
}
