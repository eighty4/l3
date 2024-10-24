use crate::code::parse::imports::node::NodeImportResolver;
use crate::code::parse::imports::ImportResolver;
use crate::code::runtime::node::NodeConfig;
use crate::code::source::ModuleImport;
use crate::testing::project::ProjectTest;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::test]
async fn test_node_import_resolver_with_relative_mjs_import() {
    let project_test = ProjectTest::with_file("lib.mjs", "");
    let source_path = project_test.source_path("main.mjs");
    let resolver = NodeImportResolver::new(Arc::new(NodeConfig::default()));
    match resolver.resolve(&source_path, "./lib.mjs") {
        ModuleImport::RelativeSource(matched) => {
            assert_eq!(matched.rel(), &PathBuf::from("lib.mjs"))
        }
        _ => panic!(),
    }
}

#[tokio::test]
async fn test_node_import_resolver_with_relative_js_import() {
    let project_test = ProjectTest::with_file("lib.js", "");
    let source_path = project_test.source_path("main.js");
    let resolver = NodeImportResolver::new(Arc::new(NodeConfig::default()));
    match resolver.resolve(&source_path, "./lib.js") {
        ModuleImport::RelativeSource(matched) => {
            assert_eq!(matched.rel(), &PathBuf::from("lib.js"))
        }
        _ => panic!(),
    }
}

#[tokio::test]
async fn test_node_import_resolver_with_unknown_import_without_extension() {
    let project_test = ProjectTest::with_file("lib.js", "");
    let source_path = project_test.source_path("main.js");
    let resolver = NodeImportResolver::new(Arc::new(NodeConfig::default()));
    match resolver.resolve(&source_path, "./lib") {
        ModuleImport::Unknown(_) => {}
        _ => panic!(),
    }
}

#[tokio::test]
async fn test_node_import_resolver_with_unknown_import_without_file_existing() {
    let project_test = ProjectTest::builder().build();
    let source_path = project_test.source_path("main.js");
    let resolver = NodeImportResolver::new(Arc::new(NodeConfig::default()));
    match resolver.resolve(&source_path, "./lib.js") {
        ModuleImport::Unknown(_) => {}
        _ => panic!(),
    }
}
