use crate::code::source::ModuleImports;
use crate::testing::project::ProjectTest;

#[tokio::test]
async fn test_parse_source_file_parses_es_export_arrow_fn() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "export const DELETE = () => {}");
        let module = project_test.source_file(path);
        assert!(module.exported_fns.contains(&"DELETE".to_string()));
        assert!(matches!(module.imports, ModuleImports::Empty));
    }
}

#[tokio::test]
async fn test_parse_source_file_parses_es_export_fn() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "export function DELETE () {}");
        let module = project_test.source_file(path);
        assert!(module.exported_fns.contains(&"DELETE".to_string()));
        assert!(matches!(module.imports, ModuleImports::Empty));
    }
}

#[tokio::test]
async fn test_parse_source_file_parses_es_export_fn_assignment() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "export const DELETE = function () {}");
        let module = project_test.source_file(path);
        assert!(module.exported_fns.contains(&"DELETE".to_string()));
        assert!(matches!(module.imports, ModuleImports::Empty));
    }
}

#[tokio::test]
async fn test_parse_source_file_parse_es_errors_for_ts_syntax() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "export function DELETE (e: string) {}");
        assert!(project_test
            .parse_result(path)
            .err()
            .unwrap()
            .to_string()
            .contains("Expected ',', got ':'"));
    }
}

#[tokio::test]
async fn test_parse_source_file_parses_es_import_module() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test =
            ProjectTest::with_file(path, "import {dbHelper} from '../db.js'\nconst string = ''");
        let module = project_test.source_file(path);
        assert!(module.exported_fns.is_empty());
        match module.imports {
            ModuleImports::Unprocessed(imports) => {
                assert_eq!(imports, vec!["../db.js".to_string()]);
            }
            _ => panic!(),
        }
    }
}

#[tokio::test]
async fn test_parse_source_file_errors_for_cjs() {
    let project_test =
        ProjectTest::with_file("lambda.js", "module.exports = {PUT: function () {}}");
    let result = project_test.parse_result("lambda.js");
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().to_string(),
        "swc compiler error: L3 does not support CJS format used in source file lambda.js"
    );
}

#[tokio::test]
async fn test_parse_source_file_parses_ts_import_module() {
    let path = "lambda.ts";
    let project_test = ProjectTest::with_file(
        path,
        "import {dbHelper} from '../db.js'\nconst v: string = ''",
    );
    let module = project_test.source_file(path);
    assert!(module.exported_fns.is_empty());
    match module.imports {
        ModuleImports::Unprocessed(imports) => {
            assert_eq!(imports, vec!["../db.js".to_string()]);
        }
        _ => panic!(),
    }
}

#[tokio::test]
async fn test_parse_source_file_parses_ts_export_arrow_fn() {
    let path = "lambda.ts";
    let project_test = ProjectTest::with_file(path, "export const DELETE = (e: string) => {}");
    let module = project_test.source_file(path);
    assert!(module.exported_fns.contains(&"DELETE".to_string()));
    assert!(matches!(module.imports, ModuleImports::Empty));
}

#[tokio::test]
async fn test_parse_source_file_parses_ts_export_fn() {
    let path = "lambda.ts";
    let project_test = ProjectTest::with_file(path, "export function DELETE (e: string) {}");
    let module = project_test.source_file(path);
    assert!(module.exported_fns.contains(&"DELETE".to_string()));
    assert!(matches!(module.imports, ModuleImports::Empty));
}

#[tokio::test]
async fn test_parse_source_file_parses_ts_export_fn_assignment() {
    let path = "lambda.ts";
    let project_test =
        ProjectTest::with_file(path, "export const DELETE = function (e: string) {}");
    let module = project_test.source_file(path);
    assert!(module.exported_fns.contains(&"DELETE".to_string()));
    assert!(matches!(module.imports, ModuleImports::Empty));
}

#[tokio::test]
async fn test_parse_source_file_surfaces_error_from_compiler() {
    let project_test = ProjectTest::with_file("lambda.js", "");
    assert!(project_test.parse_result("lambda.js").is_err());
}
