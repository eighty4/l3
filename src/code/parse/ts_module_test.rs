use std::fs;

use temp_dir::TempDir;

use crate::code::parse::parse_module_for_lambda_handlers;

#[test]
fn test_parse_ts_module_not_yet_supported() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("lambda.ts");
    fs::write(&module_path, "export function GET() {}").unwrap();
    assert!(parse_module_for_lambda_handlers(&module_path)
        .unwrap_err()
        .to_string()
        .starts_with("ts is not a supported file type for source file "));
}
