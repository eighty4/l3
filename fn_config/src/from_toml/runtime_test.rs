use crate::{configs::*, NodeVersion, PythonVersion};
use crate::{ConfigParseError, OptionallyParsable};

#[test]
fn test_neither_runtimes_configured() {
    assert_eq!(
        LambdaRuntimeConfig::optionally_parse(&"".parse::<toml::Table>().unwrap()),
        Ok(None)
    );
}

#[test]
fn test_both_node_versions_errors_when_mismatched() {
    let s = r#"
node_version = 22
[node_opts]
version = 20
"#;
    assert_eq!(
        LambdaRuntimeConfig::optionally_parse(&s.parse::<toml::Table>().unwrap()),
        Err(ConfigParseError::Misconfigured {
            actual: "node_version and node_opts.version do not match".into(),
            expected: "use node_version or node_opts.version".into(),
            field: "node_opts.version".into(),
        })
    );
}

#[test]
fn test_both_node_versions_matching() {
    let s = r#"
node_version = 22
[node_opts]
version = 22
"#;
    assert_eq!(
        LambdaRuntimeConfig::optionally_parse(&s.parse::<toml::Table>().unwrap()),
        Ok(Some(LambdaRuntimeConfig {
            node: Some(NodeLambdaConfig {
                version: Some(NodeVersion::TwentyTwo),
            }),
            python: None,
        }))
    );
}

#[test]
fn test_node_version() {
    let s = r#"
node_version = 22
"#;
    assert_eq!(
        LambdaRuntimeConfig::optionally_parse(&s.parse::<toml::Table>().unwrap()),
        Ok(Some(LambdaRuntimeConfig {
            node: Some(NodeLambdaConfig {
                version: Some(NodeVersion::TwentyTwo),
            }),
            python: None,
        }))
    );
}

#[test]
fn test_node_opts_node_version() {
    let s = r#"
[node_opts]
version = 22
"#;
    assert_eq!(
        LambdaRuntimeConfig::optionally_parse(&s.parse::<toml::Table>().unwrap()),
        Ok(Some(LambdaRuntimeConfig {
            node: Some(NodeLambdaConfig {
                version: Some(NodeVersion::TwentyTwo),
            }),
            python: None,
        }))
    );
}

#[test]
fn test_both_python_versions_errors_when_mismatched() {
    let s = r#"
python_version = 3.13
[python_opts]
version = 3.14
"#;
    assert_eq!(
        LambdaRuntimeConfig::optionally_parse(&s.parse::<toml::Table>().unwrap()),
        Err(ConfigParseError::Misconfigured {
            actual: "python_version and python_opts.version do not match".into(),
            expected: "use python_version or python_opts.version".into(),
            field: "python_opts.version".into(),
        })
    );
}

#[test]
fn test_both_python_versions_matching() {
    let s = r#"
python_version = 3.14
[python_opts]
version = 3.14
"#;
    assert_eq!(
        LambdaRuntimeConfig::optionally_parse(&s.parse::<toml::Table>().unwrap()),
        Ok(Some(LambdaRuntimeConfig {
            node: None,
            python: Some(PythonLambdaConfig {
                version: Some(PythonVersion::ThreeFourteen),
            }),
        }))
    );
}

#[test]
fn test_python_version() {
    let s = r#"
python_version = 3.14
"#;
    assert_eq!(
        LambdaRuntimeConfig::optionally_parse(&s.parse::<toml::Table>().unwrap()),
        Ok(Some(LambdaRuntimeConfig {
            node: None,
            python: Some(PythonLambdaConfig {
                version: Some(PythonVersion::ThreeFourteen),
            }),
        }))
    );
}

#[test]
fn test_python_opts_python_version() {
    let s = r#"
[python_opts]
version = 3.14
"#;
    assert_eq!(
        LambdaRuntimeConfig::optionally_parse(&s.parse::<toml::Table>().unwrap()),
        Ok(Some(LambdaRuntimeConfig {
            node: None,
            python: Some(PythonLambdaConfig {
                version: Some(PythonVersion::ThreeFourteen),
            }),
        }))
    );
}

#[test]
fn test_both_runtimes_configured() {
    let s = r#"
[node_opts]
version = 22
[python_opts]
version = 3.14
"#;
    assert_eq!(
        LambdaRuntimeConfig::optionally_parse(&s.parse::<toml::Table>().unwrap()),
        Ok(Some(LambdaRuntimeConfig {
            node: Some(NodeLambdaConfig {
                version: Some(NodeVersion::TwentyTwo),
            }),
            python: Some(PythonLambdaConfig {
                version: Some(PythonVersion::ThreeFourteen),
            }),
        }))
    );
}
