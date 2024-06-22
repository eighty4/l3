use crate::code::parse::parse_module_for_lambda_handlers;
use std::fs;
use temp_dir::TempDir;

#[test]
fn test_parse_module_errors_for_multiple_handlers() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("lambda.js");
    fs::write(
        &module_path,
        "export function GET() {}\nexport function get() {}",
    )
    .unwrap();
    assert!(parse_module_for_lambda_handlers(&module_path)
        .unwrap_err()
        .to_string()
        .starts_with("multiple GET functions found in source file "));
}

#[test]
fn test_parse_module_errors_for_unsupported_source_type() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("lambda.py");
    fs::write(&module_path, "def GET():\n    \n\n").unwrap();
    assert!(parse_module_for_lambda_handlers(&module_path)
        .unwrap_err()
        .to_string()
        .starts_with("py is not a supported file type for source file "));
}

#[test]
fn test_parse_module_errors_for_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("lambda.js");
    assert!(parse_module_for_lambda_handlers(&module_path)
        .unwrap_err()
        .to_string()
        .starts_with("source file does not exist at "));
}

#[test]
fn test_parse_module_errors_for_missing_file_extension() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("lambda");
    fs::write(
        &module_path,
        "export function GET() {}\nexport function get() {}",
    )
    .unwrap();
    assert!(parse_module_for_lambda_handlers(&module_path)
        .unwrap_err()
        .to_string()
        .starts_with("file extension missing for source file "));
}
