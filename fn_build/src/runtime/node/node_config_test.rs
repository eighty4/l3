use crate::runtime::node::{NodeConfig, NodeConfigError};

#[test]
pub fn test_parse_node_config_with_dependencies() {
    let node_config =
        NodeConfig::parse_node_config(r##"{"dependencies":{"data-lib":"0.0.1"}}"##).unwrap();
    assert!(node_config.has_npm_dependency(&"data-lib".to_string()));
    assert!(!node_config.has_npm_dependency(&"logging-lib".to_string()));
}

#[test]
pub fn test_parse_node_config_with_subpath_imports() {
    let node_config =
        NodeConfig::parse_node_config(r##"{"imports":{"#lib":{"node":"./lib.js"}}}"##).unwrap();
    assert!(!node_config.subpath_imports.is_empty());
}

#[test]
pub fn test_parse_node_config_without_subpath_imports() {
    let node_config = NodeConfig::parse_node_config("{}").unwrap();
    assert!(node_config.subpath_imports.is_empty());
}

#[test]
pub fn test_parse_node_config_with_invalid_subpath_imports() {
    match NodeConfig::parse_node_config(r##"{"imports":{"#lib":{"nick":"./lib.js"}}}"##) {
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
pub fn test_parse_node_config_resolves_package_type() {
    for (package_json, expected) in [("{}", false), (r#"{"type":"module"}"#, true)] {
        let node_config = NodeConfig::parse_node_config(package_json).unwrap();
        assert_eq!(
            node_config.is_es_module(),
            expected,
            "package.json={package_json} should produce node_config.module_type={expected}"
        );
    }
}
