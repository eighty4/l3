use crate::code::parse::imports::typescript::TypescriptImportResolver;
use crate::code::parse::imports::ImportResolver;
use crate::code::source::ModuleImport;
use crate::testing::project::ProjectTest;
use std::path::PathBuf;

#[tokio::test]
async fn test_typescript_import_resolver_with_relative_esm_js_import() {
    let project_test = ProjectTest::with_file("lib.ts", "");
    let source_path = project_test.source_path("main.ts");
    let resolver = TypescriptImportResolver::new();
    match resolver.resolve(&source_path, "./lib.js") {
        ModuleImport::RelativeSource(matched) => {
            assert_eq!(matched.rel(), &PathBuf::from("lib.ts"))
        }
        _ => panic!(),
    }
}

#[tokio::test]
async fn test_typescript_import_resolver_with_unknown_import_with_ts_extension() {
    let project_test = ProjectTest::with_file("lib.ts", "");
    let source_path = project_test.source_path("main.ts");
    let resolver = TypescriptImportResolver::new();
    match resolver.resolve(&source_path, "./lib.ts") {
        ModuleImport::Unknown(unresolved) => assert_eq!(unresolved, "./lib.ts".to_string()),
        _ => panic!(),
    }
}

#[tokio::test]
async fn test_typescript_import_resolver_with_unknown_import_without_extension() {
    let project_test = ProjectTest::with_file("lib.ts", "");
    let source_path = project_test.source_path("main.ts");
    let resolver = TypescriptImportResolver::new();
    match resolver.resolve(&source_path, "./lib") {
        ModuleImport::Unknown(unresolved) => assert_eq!(unresolved, "./lib".to_string()),
        _ => panic!(),
    }
}

#[tokio::test]
async fn test_typescript_import_resolver_with_unknown_import_without_file_existing() {
    let project_test = ProjectTest::builder().build();
    let source_path = project_test.source_path("main.ts");
    let resolver = TypescriptImportResolver::new();
    match resolver.resolve(&source_path, "./lib.js") {
        ModuleImport::Unknown(unresolved) => assert_eq!(unresolved, "./lib.js".to_string()),
        _ => panic!(),
    }
}
