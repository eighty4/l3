use std::fs;

use temp_dir::TempDir;

use crate::runtime::node::{NodeConfig, NodeConfigError};

#[test]
pub fn test_parse_node_config_reads_tsconfig() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.child("package.json"), r##"{}"##).unwrap();
    fs::write(
        temp_dir.child("tsconfig.json"),
        r#"{
    "compilerOptions": {
        "allowImportingTsExtensions": true,
        "allowJs": true,
        "rewriteRelativeImportExtensions": true
    }
}
"#,
    )
    .unwrap();
    let node_config = NodeConfig::read_configs(temp_dir.path()).unwrap();
    assert!(node_config.ts.is_some());
}

#[test]
pub fn test_parse_node_config_without_tsconfig() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.child("package.json"), r##"{}"##).unwrap();
    let node_config = NodeConfig::read_configs(temp_dir.path()).unwrap();
    assert!(node_config.ts.is_none());
}

#[test]
pub fn test_parse_node_config_with_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.child("package.json"),
        r##"{"dependencies":{"data-lib":"0.0.1"}}"##,
    )
    .unwrap();
    let node_config = NodeConfig::read_configs(temp_dir.path()).unwrap();
    assert!(node_config
        .package
        .has_npm_dependency(&"data-lib".to_string()));
    assert!(!node_config
        .package
        .has_npm_dependency(&"logging-lib".to_string()));
}

#[test]
pub fn test_parse_node_config_with_subpath_imports() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.child("package.json"),
        r##"{"imports":{"#lib":{"node":"./lib.js"}}}"##,
    )
    .unwrap();
    let node_config = NodeConfig::read_configs(temp_dir.path()).unwrap();
    assert!(!node_config.package.subpath_imports.is_empty());
}

#[test]
pub fn test_parse_node_config_without_subpath_imports() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.child("package.json"), "{}").unwrap();
    let node_config = NodeConfig::read_configs(temp_dir.path()).unwrap();
    assert!(node_config.package.subpath_imports.is_empty());
}

#[test]
pub fn test_parse_node_config_with_invalid_subpath_imports() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.child("package.json"),
        r##"{"imports":{"#lib":{"nick":"./lib.js"}}}"##,
    )
    .unwrap();
    match NodeConfig::read_configs(temp_dir.path()) {
        Err(NodeConfigError::SubpathImport(err)) => {
            assert_eq!(
                err.to_string(),
                "subpath import condition `nick` is invalid".to_string()
            );
        }
        _ => panic!(),
    }
}

#[test]
pub fn test_parse_node_config_resolves_module_package_type() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.child("package.json"), r#"{"type":"module"}"#).unwrap();
    let node_config = NodeConfig::read_configs(temp_dir.path()).unwrap();
    assert!(node_config.package.is_es_module());
}

#[test]
pub fn test_parse_node_config_resolves_not_module_package_type_when_unspecified() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.child("package.json"), "{}").unwrap();
    let node_config = NodeConfig::read_configs(temp_dir.path()).unwrap();
    assert!(!node_config.package.is_es_module());
}
