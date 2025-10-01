use std::collections::HashMap;

use crate::configs::*;

#[test]
fn test_parse_l3_toml_with_single_lambda_table() {
    let s = r#"
[lambda]
name = "data-endpoint-fn"
"#;
    let project_v = s.parse::<toml::Table>().unwrap();
    assert_eq!(
        ProjectConfig::try_from(&project_v),
        Ok(ProjectConfig {
            lambdas: vec![LambdaConfig {
                name: Some("data-endpoint-fn".into()),
                ..Default::default()
            }],
            ..Default::default()
        })
    );
}

#[test]
fn test_parse_l3_toml_with_lambda_array() {
    let s = r#"
[[lambda]]
name = "data-fn"

[[lambda]]
name = "more-data-fn"
"#;
    let project_v = s.parse::<toml::Table>().unwrap();
    assert_eq!(
        ProjectConfig::try_from(&project_v),
        Ok(ProjectConfig {
            lambdas: vec![
                LambdaConfig {
                    name: Some("data-fn".into()),
                    ..Default::default()
                },
                LambdaConfig {
                    name: Some("more-data-fn".into()),
                    ..Default::default()
                }
            ],
            ..Default::default()
        })
    );
}

#[test]
fn test_parse_l3_toml_with_project_env_vars_table() {
    let s = r#"
[env_vars]
API_CLIENT_ID="123"
API_CLIENT_SECRET="im a secret"
"#;
    let project_v = s.parse::<toml::Table>().unwrap();
    assert_eq!(
        ProjectConfig::try_from(&project_v),
        Ok(ProjectConfig {
            env: Some(LambdaEnvConfig {
                env_files: None,
                env_vars: Some(HashMap::from([
                    ("API_CLIENT_ID".into(), "123".into()),
                    ("API_CLIENT_SECRET".into(), "im a secret".into())
                ]))
            }),
            ..Default::default()
        })
    );
}

#[test]
fn test_parse_l3_toml_with_lambda_env_vars_table() {
    let s = r#"
[[lambda]]
name = "data-fn"
[lambda.env_vars]
API_CLIENT_ID="123"
API_CLIENT_SECRET="im a secret"
"#;
    let project_v = s.parse::<toml::Table>().unwrap();
    assert_eq!(
        ProjectConfig::try_from(&project_v),
        Ok(ProjectConfig {
            lambdas: vec![LambdaConfig {
                name: Some("data-fn".into()),
                env: Some(LambdaEnvConfig {
                    env_files: None,
                    env_vars: Some(HashMap::from([
                        ("API_CLIENT_ID".into(), "123".into()),
                        ("API_CLIENT_SECRET".into(), "im a secret".into())
                    ]))
                }),
                ..Default::default()
            }],
            ..Default::default()
        })
    );
}

#[test]
fn test_parse_l3_toml_with_runtime_config() {
    let s = r#"
node_version = 22
[python_opts]
version = 3.14
"#;
    assert_eq!(
        ProjectConfig::try_from(&s.parse::<toml::Table>().unwrap()),
        Ok(ProjectConfig {
            runtime: Some(LambdaRuntimeConfig {
                node: Some(NodeLambdaConfig {
                    version: Some(NodeVersion::TwentyTwo),
                }),
                python: Some(PythonLambdaConfig {
                    version: Some(PythonVersion::ThreeFourteen),
                }),
            }),
            ..Default::default()
        })
    );
}
