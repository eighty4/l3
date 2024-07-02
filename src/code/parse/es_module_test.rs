use std::path::PathBuf;

use crate::code::parse::parse_source_file;
use crate::testing::ProjectTest;

#[test]
fn test_es_module_parse_source_file_parses_import_module() {
    let project_test = ProjectTest::with_file("lambda.mjs", "import {dbHelper} from '../db.js'");
    let module =
        parse_source_file(&PathBuf::from("lambda.mjs"), &project_test.project_dir).unwrap();
    assert!(module.imports.contains(&PathBuf::from("../db.js")));
    assert!(module.exported_fns.is_empty());
}

#[test]
fn test_es_module_parse_source_file_parses_export_arrow_fn() {
    let project_test = ProjectTest::with_file("lambda.mjs", "export const DELETE = () => {}");
    let module =
        parse_source_file(&PathBuf::from("lambda.mjs"), &project_test.project_dir).unwrap();
    assert!(module.imports.is_empty());
    assert!(module.exported_fns.contains(&"DELETE".to_string()));
}

#[test]
fn test_es_module_parse_source_file_parses_export_fn() {
    let project_test = ProjectTest::with_file("lambda.mjs", "export function DELETE () {}");
    let module =
        parse_source_file(&PathBuf::from("lambda.mjs"), &project_test.project_dir).unwrap();
    assert!(module.imports.is_empty());
    assert!(module.exported_fns.contains(&"DELETE".to_string()));
}

#[test]
fn test_es_module_parse_source_file_errors_for_cjs() {
    let project_test =
        ProjectTest::with_file("lambda.js", "module.exports = {PUT: function () {}}");
    assert!(
        parse_source_file(&PathBuf::from("lambda.js"), &project_test.project_dir)
            .err()
            .unwrap()
            .to_string()
            .starts_with("unable to parse CJS format for source file ")
    );
}

#[test]
fn test_es_module_parse_source_file_surfaces_error_from_compiler() {
    let project_test = ProjectTest::with_file("lambda.js", "");
    assert!(parse_source_file(&PathBuf::from("lambda.js"), &project_test.project_dir).is_err());
}
