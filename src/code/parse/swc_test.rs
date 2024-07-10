use std::path::PathBuf;

use crate::code::parse::parse_source_file;
use crate::testing::ProjectTest;

#[test]
fn test_parse_source_file_parses_es_import_module() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "import {dbHelper} from '../db.js'");
        let module = parse_source_file(&PathBuf::from(path), &project_test.project_dir).unwrap();
        assert!(module.imports.contains(&PathBuf::from("../db.js")));
        assert!(module.exported_fns.is_empty());
    }
}

#[test]
fn test_parse_source_file_parses_es_export_arrow_fn() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "export const DELETE = () => {}");
        let module = parse_source_file(&PathBuf::from(path), &project_test.project_dir).unwrap();
        assert!(module.imports.is_empty());
        assert!(module.exported_fns.contains(&"DELETE".to_string()));
    }
}

#[test]
fn test_parse_source_file_parses_es_export_fn() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "export function DELETE () {}");
        let module = parse_source_file(&PathBuf::from(path), &project_test.project_dir).unwrap();
        assert!(module.imports.is_empty());
        assert!(module.exported_fns.contains(&"DELETE".to_string()));
    }
}

#[test]
fn test_parse_source_file_parse_es_errors_for_ts_syntax() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "export function DELETE (e: string) {}");
        assert!(
            parse_source_file(&PathBuf::from(path), &project_test.project_dir)
                .err()
                .unwrap()
                .to_string()
                .contains("Expected ',', got ':'")
        );
    }
}

#[test]
fn test_parse_source_file_errors_for_cjs() {
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
fn test_parse_source_file_parses_ts_import_module() {
    let path = "lambda.ts";
    let project_test = ProjectTest::with_file(
        path,
        "import {dbHelper} from '../db.js'\nconst v: string = ''",
    );
    let module = parse_source_file(&PathBuf::from(path), &project_test.project_dir).unwrap();
    assert!(module.imports.contains(&PathBuf::from("../db.js")));
    assert!(module.exported_fns.is_empty());
}

#[test]
fn test_parse_source_file_parses_ts_export_arrow_fn() {
    let path = "lambda.ts";
    let project_test = ProjectTest::with_file(path, "export const DELETE = (e: string) => {}");
    let module = parse_source_file(&PathBuf::from(path), &project_test.project_dir).unwrap();
    assert!(module.imports.is_empty());
    assert!(module.exported_fns.contains(&"DELETE".to_string()));
}

#[test]
fn test_parse_source_file_parses_ts_export_fn() {
    let path = "lambda.ts";
    let project_test = ProjectTest::with_file(path, "export function DELETE (e: string) {}");
    let module = parse_source_file(&PathBuf::from(path), &project_test.project_dir).unwrap();
    assert!(module.imports.is_empty());
    assert!(module.exported_fns.contains(&"DELETE".to_string()));
}

#[test]
fn test_parse_source_file_surfaces_error_from_compiler() {
    let project_test = ProjectTest::with_file("lambda.js", "");
    assert!(parse_source_file(&PathBuf::from("lambda.js"), &project_test.project_dir).is_err());
}
