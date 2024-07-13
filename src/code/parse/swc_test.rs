use std::path::{Path, PathBuf};

use crate::code::parse::parse_source_file;
use crate::code::project::ProjectDetails;
use crate::code::source::ModuleImport;
use crate::code::source::ModuleImport::*;
use crate::testing::ProjectTest;

fn verify_relative_source_import(import: &ModuleImport, project_dir: &Path, path: &str) {
    if let RelativeSource(import_path) = import {
        assert_eq!(import_path.abs, project_dir.join(path));
        assert_eq!(import_path.rel, PathBuf::from(path));
    } else {
        panic!();
    }
}

fn verify_package_dependency_import(
    import: &ModuleImport,
    expected_package: String,
    expected_subpath: Option<String>,
) {
    if let PackageDependency { package, subpath } = import {
        assert_eq!(package, &expected_package);
        match expected_subpath {
            None => assert!(subpath.is_none()),
            Some(s) => {
                assert!(subpath.is_some());
                assert_eq!(s, subpath.clone().unwrap())
            }
        }
    } else {
        panic!();
    }
}

fn verify_unknown_import(import: &ModuleImport, expected_value: String) {
    if let Unknown(value) = import {
        assert_eq!(value, &expected_value);
    } else {
        panic!();
    }
}

fn verify_node_subpath_import(import: &ModuleImport) {
    if let NodeSubpathImport { declared, path } = import {
    } else {
        panic!();
    }
}

#[test]
fn test_parse_source_file_parses_es_package_dependency_import_without_subpath() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "import {dbHelper} from 'db-dep'");
        let mut project_details = ProjectDetails::default();
        project_details
            .javascript
            .dependencies
            .push("db-dep".to_string());
        let module = parse_source_file(project_test.source_path(path), &project_details).unwrap();
        assert_eq!(module.imports.len(), 1);
        verify_package_dependency_import(
            module.imports.first().unwrap(),
            "db-dep".to_string(),
            None,
        );
        assert!(module.exported_fns.is_empty());
    }
}

#[test]
fn test_parse_source_file_parses_es_package_dependency_import_with_subpath() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "import {dbHelper} from 'db-dep/helpers'");
        let mut project_details = ProjectDetails::default();
        project_details
            .javascript
            .dependencies
            .push("db-dep".to_string());
        let module = parse_source_file(project_test.source_path(path), &project_details).unwrap();
        assert_eq!(module.imports.len(), 1);
        verify_package_dependency_import(
            module.imports.first().unwrap(),
            "db-dep".to_string(),
            Some("helpers".to_string()),
        );
        assert!(module.exported_fns.is_empty());
    }
}

#[test]
fn test_parse_source_file_parses_es_package_dependency_import_as_unknown_without_dependency() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "import {dbHelper} from 'db-dep'");
        let module =
            parse_source_file(project_test.source_path(path), &Default::default()).unwrap();
        assert_eq!(module.imports.len(), 1);
        verify_unknown_import(module.imports.first().unwrap(), "db-dep".to_string());
        assert!(module.exported_fns.is_empty());
    }
}

#[test]
fn test_parse_source_file_parses_es_node_subpath_import() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "import {dbHelper} from '#db/helpers.js'");
        let mut project_details = ProjectDetails::default();
        project_details
            .javascript
            .dependencies
            .push("#db-dep".to_string());
        let module = parse_source_file(project_test.source_path(path), &project_details).unwrap();
        assert_eq!(module.imports.len(), 1);
        verify_node_subpath_import(module.imports.first().unwrap());
        assert!(module.exported_fns.is_empty());
    }
}

#[test]
fn test_parse_source_file_parses_es_node_subpath_import_without_alias() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "import {dbHelper} from '#db/helpers.js'");
        let module =
            parse_source_file(project_test.source_path(path), &Default::default()).unwrap();
        assert_eq!(module.imports.len(), 1);
        verify_unknown_import(
            module.imports.first().unwrap(),
            "#db/helpers.js".to_string(),
        );
        assert!(module.exported_fns.is_empty());
    }
}

#[test]
fn test_parse_source_file_parses_es_relative_source_import() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "import {dbHelper} from '../db.js'");
        let module =
            parse_source_file(project_test.source_path(path), &Default::default()).unwrap();
        assert_eq!(module.imports.len(), 1);
        verify_relative_source_import(
            module.imports.first().unwrap(),
            &project_test.project_dir,
            "../db.js",
        );
        assert!(module.exported_fns.is_empty());
    }
}

#[test]
fn test_parse_source_file_parses_es_export_arrow_fn() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "export const DELETE = () => {}");
        let module =
            parse_source_file(project_test.source_path(path), &Default::default()).unwrap();
        assert!(module.imports.is_empty());
        assert!(module.exported_fns.contains(&"DELETE".to_string()));
    }
}

#[test]
fn test_parse_source_file_parses_es_export_fn() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "export function DELETE () {}");
        let module =
            parse_source_file(project_test.source_path(path), &Default::default()).unwrap();
        assert!(module.imports.is_empty());
        assert!(module.exported_fns.contains(&"DELETE".to_string()));
    }
}

#[test]
fn test_parse_source_file_parse_es_errors_for_ts_syntax() {
    for path in &["lambda.js", "lambda.mjs"] {
        let project_test = ProjectTest::with_file(path, "export function DELETE (e: string) {}");
        assert!(
            parse_source_file(project_test.source_path(path), &Default::default())
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
        parse_source_file(project_test.source_path("lambda.js"), &Default::default())
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
    let module = parse_source_file(project_test.source_path(path), &Default::default()).unwrap();
    assert_eq!(module.imports.len(), 1);
    verify_relative_source_import(
        module.imports.first().unwrap(),
        &project_test.project_dir,
        "../db.js",
    );
    assert!(module.exported_fns.is_empty());
}

#[test]
fn test_parse_source_file_parses_ts_export_arrow_fn() {
    let path = "lambda.ts";
    let project_test = ProjectTest::with_file(path, "export const DELETE = (e: string) => {}");
    let module = parse_source_file(project_test.source_path(path), &Default::default()).unwrap();
    assert!(module.imports.is_empty());
    assert!(module.exported_fns.contains(&"DELETE".to_string()));
}

#[test]
fn test_parse_source_file_parses_ts_export_fn() {
    let path = "lambda.ts";
    let project_test = ProjectTest::with_file(path, "export function DELETE (e: string) {}");
    let module = parse_source_file(project_test.source_path(path), &Default::default()).unwrap();
    assert!(module.imports.is_empty());
    assert!(module.exported_fns.contains(&"DELETE".to_string()));
}

#[test]
fn test_parse_source_file_surfaces_error_from_compiler() {
    let project_test = ProjectTest::with_file("lambda.js", "");
    assert!(parse_source_file(project_test.source_path("lambda.js"), &Default::default()).is_err());
}
