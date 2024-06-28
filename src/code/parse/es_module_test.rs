use std::fs;
use std::path::PathBuf;
use temp_dir::TempDir;

use crate::code::parse::es_module::EsModule;
use crate::code::parse::parse_module_for_lambda_handlers;
use crate::lambda::HttpMethod;

#[test]
fn test_es_module_parse_import_module() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("lambda.mjs");
    fs::write(&module_path, "import {dbHelper} from '../db.js'").unwrap();
    let module = EsModule::parse(module_path.as_path()).unwrap();
    assert!(module.module_imports.contains(&PathBuf::from("../db.js")));
    assert!(module.exported_fns.is_empty());
}

#[test]
fn test_es_module_parse_export_arrow_fn() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("lambda.mjs");
    fs::write(&module_path, "export const DELETE = () => {}").unwrap();
    let module = EsModule::parse(module_path.as_path()).unwrap();
    assert!(module.module_imports.is_empty());
    assert!(module.exported_fns.contains(&"DELETE".to_string()));
}

#[test]
fn test_es_module_parse_export_fn() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("lambda.mjs");
    fs::write(&module_path, "export function DELETE () {}").unwrap();
    let module = EsModule::parse(module_path.as_path()).unwrap();
    assert!(module.module_imports.is_empty());
    assert!(module.exported_fns.contains(&"DELETE".to_string()));
}

#[test]
fn test_parse_js_module_finds_delete_fn() {
    for module_file_name in &["lambda.js", "lambda.mjs"] {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join(module_file_name);
        fs::write(&module_path, "export function DELETE() {}").unwrap();
        let handlers = parse_module_for_lambda_handlers(&module_path).unwrap();
        assert_eq!(handlers.len(), 1);
        assert!(handlers.contains_key(&HttpMethod::Delete));
        assert_eq!(handlers.get(&HttpMethod::Delete).unwrap(), "DELETE");
    }
}

#[test]
fn test_parse_module_finds_get_fn() {
    for module_file_name in &["lambda.js", "lambda.mjs"] {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join(module_file_name);
        fs::write(&module_path, "export function GET() {}").unwrap();
        let handlers = parse_module_for_lambda_handlers(&module_path).unwrap();
        assert_eq!(handlers.len(), 1);
        assert!(handlers.contains_key(&HttpMethod::Get));
        assert_eq!(handlers.get(&HttpMethod::Get).unwrap(), "GET");
    }
}

#[test]
fn test_parse_module_finds_patch_fn() {
    for module_file_name in &["lambda.js", "lambda.mjs"] {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join(module_file_name);
        fs::write(&module_path, "export function PATCH() {}").unwrap();
        let handlers = parse_module_for_lambda_handlers(&module_path).unwrap();
        assert_eq!(handlers.len(), 1);
        assert!(handlers.contains_key(&HttpMethod::Patch));
        assert_eq!(handlers.get(&HttpMethod::Patch).unwrap(), "PATCH");
    }
}

#[test]
fn test_parse_module_finds_post_fn() {
    for module_file_name in &["lambda.js", "lambda.mjs"] {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join(module_file_name);
        fs::write(&module_path, "export function POST() {}").unwrap();
        let handlers = parse_module_for_lambda_handlers(&module_path).unwrap();
        assert_eq!(handlers.len(), 1);
        assert!(handlers.contains_key(&HttpMethod::Post));
        assert_eq!(handlers.get(&HttpMethod::Post).unwrap(), "POST");
    }
}

#[test]
fn test_parse_module_finds_put_fn() {
    for module_file_name in &["lambda.js", "lambda.mjs"] {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join(module_file_name);
        fs::write(&module_path, "export function PUT() {}").unwrap();
        let handlers = parse_module_for_lambda_handlers(&module_path).unwrap();
        assert_eq!(handlers.len(), 1);
        assert!(handlers.contains_key(&HttpMethod::Put));
        assert_eq!(handlers.get(&HttpMethod::Put).unwrap(), "PUT");
    }
}

#[test]
fn test_parse_module_errors_for_cjs() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("lambda.js");
    fs::write(&module_path, "module.exports = {PUT: function () {}}").unwrap();
    assert!(parse_module_for_lambda_handlers(&module_path)
        .unwrap_err()
        .to_string()
        .starts_with("unable to parse CJS format for source file "));
}

#[test]
fn test_es_module_parse_surfaces_error_from_compiler() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("lambda.js");
    assert!(EsModule::parse(&module_path).is_err());
}
