use crate::code::runtime::node::NodePackageManager::{Npm, Pnpm};
use crate::code::runtime::node::{read_node_config, NodeConditionalImportKind};
use crate::testing::project::ProjectTest;
use crate::testing::source::TestSource;
use std::path::PathBuf;

#[tokio::test]
async fn test_read_node_config_defaults_when_package_json_does_not_exist() {
    let project_test = ProjectTest::builder().build();
    let node_config = read_node_config(&project_test.project_dir);
    assert!(node_config.dependencies.is_empty());
    assert!(matches!(node_config.package_manager, Npm));
    assert!(node_config.subpath_imports.is_empty());
}

#[tokio::test]
async fn test_read_node_config_defaults_when_package_json_does_not_exist_with_pnpm() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("pnpm-lock.yaml").content(""))
        .build();
    let node_config = read_node_config(&project_test.project_dir);
    assert!(node_config.dependencies.is_empty());
    assert!(matches!(node_config.package_manager, Pnpm));
    assert!(node_config.subpath_imports.is_empty());
}

#[tokio::test]
async fn test_read_node_config_reads_dependencies() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("package.json").content(
            "{\"dependencies\":{\"pg\":\"0.0.1\"},\"devDependencies\":{\"@types/pg\":\"0.0.1\"}}",
        ))
        .build();
    let node_config = read_node_config(&project_test.project_dir);
    assert_eq!(2, node_config.dependencies.len());
    assert!(node_config.dependencies.contains(&"pg".to_string()));
    assert!(node_config.dependencies.contains(&"@types/pg".to_string()));
}

#[tokio::test]
async fn test_read_node_config_checks_for_pnpm_package_manager() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("pnpm-lock.yaml").content(""))
        .build();
    let node_config = read_node_config(&project_test.project_dir);
    assert!(matches!(node_config.package_manager, Pnpm));
}

#[tokio::test]
async fn test_read_node_config_defaults_to_npm_package_manager() {
    let project_test = ProjectTest::builder().build();
    let node_config = read_node_config(&project_test.project_dir);
    assert!(matches!(node_config.package_manager, Npm));
}

#[tokio::test]
async fn test_read_node_config_reads_subpath_imports() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("package.json").content(
            "{\"imports\":{\"#dep\":{\"node\":\"dep-node-native\",\"default\":\"./dep-polyfill.js\"}}}",
        ))
        .build();
    let node_config = read_node_config(&project_test.project_dir);
    let lookup = node_config.subpath_imports.get(&"#dep".to_string());
    assert!(lookup.is_some());
    let subpaths = lookup.unwrap();
    assert_eq!(2, subpaths.iter().len());
    let mut matches = 0;
    for subpath in subpaths {
        match subpath.kind {
            NodeConditionalImportKind::Node => {
                matches = matches + 1;
                assert_eq!(PathBuf::from("dep-node-native"), subpath.path)
            }
            NodeConditionalImportKind::Default => {
                matches = matches + 1;
                assert_eq!(PathBuf::from("./dep-polyfill.js"), subpath.path)
            }
            _ => panic!(),
        }
    }
    assert_eq!(2, matches);
}
