use std::path::PathBuf;

use crate::code::parse::es_module::EsModule;
use crate::code::parse::parse_module_for_lambda_handlers;
use crate::lambda::HttpMethod;
use crate::testing::{ProjectTest, TestSource};

#[test]
fn test_es_module_parse_import_module() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("lambda.mjs").content("import {dbHelper} from '../db.js'"),
        )
        .build();
    let module = EsModule::parse(&project_test.path("lambda.mjs")).unwrap();
    assert!(module.module_imports.contains(&PathBuf::from("../db.js")));
    assert!(module.exported_fns.is_empty());
}

#[test]
fn test_es_module_parse_export_arrow_fn() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("lambda.mjs").content("export const DELETE = () => {}"))
        .build();
    let module = EsModule::parse(&project_test.path("lambda.mjs")).unwrap();
    assert!(module.module_imports.is_empty());
    assert!(module.exported_fns.contains(&"DELETE".to_string()));
}

#[test]
fn test_es_module_parse_export_fn() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("lambda.mjs").content("export function DELETE () {}"))
        .build();
    let module = EsModule::parse(&project_test.path("lambda.mjs")).unwrap();
    assert!(module.module_imports.is_empty());
    assert!(module.exported_fns.contains(&"DELETE".to_string()));
}

#[test]
fn test_parse_js_module_finds_http_method_handler() {
    for module_file_name in &["lambda.js", "lambda.mjs"] {
        for http_method_str in ["GET", "DELETE", "PATCH", "POST", "PUT"] {
            let http_method = HttpMethod::try_from(http_method_str).unwrap();
            let project_test = ProjectTest::builder()
                .with_source(
                    TestSource::with_path(module_file_name)
                        .content(format!("export function {http_method_str}() {{}}").as_str()),
                )
                .build();
            let handlers =
                parse_module_for_lambda_handlers(&project_test.path(module_file_name)).unwrap();
            assert_eq!(handlers.len(), 1);
            assert!(handlers.contains_key(&http_method));
            assert_eq!(handlers.get(&http_method).unwrap(), http_method_str);
        }
    }
}

#[test]
fn test_parse_module_errors_for_cjs() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("lambda.js").content("module.exports = {PUT: function () {}}"),
        )
        .build();
    assert!(
        parse_module_for_lambda_handlers(&project_test.path("lambda.js"))
            .unwrap_err()
            .to_string()
            .starts_with("unable to parse CJS format for source file ")
    );
}

#[test]
fn test_es_module_parse_surfaces_error_from_compiler() {
    let project_test = ProjectTest::builder().build();
    let module_path = project_test.path("lambda.js");
    assert!(EsModule::parse(&module_path).is_err());
}
