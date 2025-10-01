use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use temp_dir::TempDir;

use crate::typescript::*;

#[test]
fn test_read_tsconfig() {
    let temp_dir = TempDir::new().unwrap();
    let p = temp_dir.child("tsconfig.json");
    fs::write(
        &p,
        r#"{
    "compilerOptions": {
        "allowImportingTsExtensions": true,
        "allowJs": true,
        "rewriteRelativeImportExtensions": true
    }
}
"#,
    )
    .unwrap();
    let tsconfig = TsConfigJson::read_tsconfig_json(&p).unwrap();
    assert_eq!(
        tsconfig,
        Arc::new(TsConfigJson {
            compiler: TsCompilerOptions {
                allow_importing_ts: true,
                allow_js: true,
                rewrite_relative_imports: true,
            }
        })
    );
}

#[test]
fn test_read_tsconfig_compiler_options_absent_values_defaulted() {
    let temp_dir = TempDir::new().unwrap();
    let p = temp_dir.child("tsconfig.json");
    fs::write(&p, r#"{}"#).unwrap();
    let tsconfig = TsConfigJson::read_tsconfig_json(&p).unwrap();
    assert_eq!(
        tsconfig,
        Arc::new(TsConfigJson {
            compiler: TsCompilerOptions {
                allow_importing_ts: false,
                allow_js: false,
                rewrite_relative_imports: false,
            }
        })
    );
}

#[test]
fn test_read_tsconfig_compiler_options_empty_values_defaulted() {
    let temp_dir = TempDir::new().unwrap();
    let p = temp_dir.child("tsconfig.json");
    fs::write(&p, r#"{"compilerOptions": {}}"#).unwrap();
    let tsconfig = TsConfigJson::read_tsconfig_json(&p).unwrap();
    assert_eq!(
        tsconfig,
        Arc::new(TsConfigJson {
            compiler: TsCompilerOptions {
                allow_importing_ts: false,
                allow_js: false,
                rewrite_relative_imports: false,
            }
        })
    );
}

#[test]
fn test_read_tsconfig_compiler_options_invalid_fields_error() {
    let mut compiler_options: HashMap<String, String> = HashMap::new();
    compiler_options.insert(
        "compilerOptions.allowImportingTsExtensions".into(),
        r#""true""#.into(),
    );
    compiler_options.insert("compilerOptions.allowJs".into(), "1".into());
    compiler_options.insert(
        "compilerOptions.rewriteRelativeImportExtensions".into(),
        r#"{"hell": "yeah"}"#.into(),
    );
    for (expect_json_path, invalid_value) in compiler_options.iter() {
        let temp_dir = TempDir::new().unwrap();
        let p = temp_dir.child("tsconfig.json");
        let json = format!(
            r#"{{"compilerOptions": {{"{}": {invalid_value}}}}}"#,
            expect_json_path.split_once(".").unwrap().1
        );
        fs::write(&p, json).unwrap();
        match TsConfigJson::read_tsconfig_json(&p) {
            Err(TsConfigError::MismatchedType {
                json_path,
                expected_type,
            }) => {
                assert_eq!(&json_path, expect_json_path);
                assert_eq!(expected_type, "bool".to_string());
            }
            _ => panic!(),
        }
    }
}
