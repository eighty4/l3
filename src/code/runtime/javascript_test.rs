use crate::code::runtime::javascript::NodePackageManager::{Npm, Pnpm};
use crate::code::runtime::javascript::{JavaScriptDeets, NodeConditionalImportKind};
use crate::testing::{ProjectTest, TestSource};
use std::path::PathBuf;

#[test]
fn test_read_details_reads_dependencies() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("package.json").content(
            "{\"dependencies\":{\"pg\":\"0.0.1\"},\"devDependencies\":{\"@types/pg\":\"0.0.1\"}}",
        ))
        .build();
    let js_deets = JavaScriptDeets::read_details(&project_test.project_dir).unwrap();
    assert_eq!(2, js_deets.dependencies.len());
    assert!(js_deets.dependencies.contains(&"pg".to_string()));
    assert!(js_deets.dependencies.contains(&"@types/pg".to_string()));
}

#[test]
fn test_read_details_checks_for_pnpm_package_manager() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("pnpm-lock.yaml").content(""))
        .build();
    let js_deets = JavaScriptDeets::read_details(&project_test.project_dir).unwrap();
    assert!(matches!(js_deets.package_manager, Pnpm));
}

#[test]
fn test_read_details_defaults_to_npm_package_manager() {
    let project_test = ProjectTest::builder().build();
    let js_deets = JavaScriptDeets::read_details(&project_test.project_dir).unwrap();
    assert!(matches!(js_deets.package_manager, Npm));
}

#[test]
fn test_read_details_reads_subpath_imports() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("package.json").content(
            "{\"imports\":{\"#dep\":{\"node\":\"dep-node-native\",\"default\":\"./dep-polyfill.js\"}}}",
        ))
        .build();
    let js_deets = JavaScriptDeets::read_details(&project_test.project_dir).unwrap();
    let lookup = js_deets.subpath_imports.get(&"#dep".to_string());
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
