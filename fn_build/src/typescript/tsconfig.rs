use std::{fs::read_to_string, io, path::Path, sync::Arc};

use serde_json::Value;

/// Errors that occur when building a TsConfigJson from tsconfig.json.
#[derive(Debug, thiserror::Error)]
#[error("error initializing package.json: {0}")]
pub enum TsConfigError {
    IoRead(#[from] io::Error),
    JsonParse(#[from] serde_json::Error),
    #[error("expected {expected_type} in json at {json_path}")]
    MismatchedType {
        json_path: String,
        expected_type: String,
    },
}

#[allow(unused)]
#[derive(Debug, Default, PartialEq)]
pub struct TsConfigJson {
    pub compiler: TsCompilerOptions,
}

#[allow(unused)]
#[derive(Debug, Default, PartialEq)]
pub struct TsCompilerOptions {
    /// Captures compilerOptions.allowImportingTsExtensions.
    pub allow_importing_ts: bool,
    /// Captures compilerOptions.allowJs.
    pub allow_js: bool,
    /// Captures compilerOptions.rewriteRelativeImportExtensions.
    pub rewrite_relative_imports: bool,
}

impl TsConfigJson {
    fn parse_tsconfig_json(tsconfig_json: &str) -> Result<Arc<Self>, TsConfigError> {
        Ok(Arc::new(Self::try_from(serde_json::from_str::<Value>(
            tsconfig_json,
        )?)?))
    }

    pub fn read_tsconfig_json(path: &Path) -> Result<Arc<Self>, TsConfigError> {
        debug_assert!(path.extension().unwrap() == "json");
        if path.is_file() {
            Self::parse_tsconfig_json(read_to_string(path)?.as_str())
        } else {
            Ok(Default::default())
        }
    }
}

impl TryFrom<Value> for TsConfigJson {
    type Error = TsConfigError;

    fn try_from(tsconfig_json: Value) -> Result<Self, Self::Error> {
        Ok(Self {
            compiler: match tsconfig_json
                .get("compilerOptions")
                .and_then(|co| co.as_object())
            {
                Some(co_json) => TsCompilerOptions {
                    allow_importing_ts: optional_bool(co_json.get("allowImportingTsExtensions"))
                        .ok_or(TsConfigError::MismatchedType {
                            json_path: "compilerOptions.allowImportingTsExtensions".into(),
                            expected_type: "bool".into(),
                        })?,
                    allow_js: optional_bool(co_json.get("allowJs")).ok_or(
                        TsConfigError::MismatchedType {
                            json_path: "compilerOptions.allowJs".into(),
                            expected_type: "bool".into(),
                        },
                    )?,
                    rewrite_relative_imports: optional_bool(
                        co_json.get("rewriteRelativeImportExtensions"),
                    )
                    .ok_or(TsConfigError::MismatchedType {
                        json_path: "compilerOptions.rewriteRelativeImportExtensions".into(),
                        expected_type: "bool".into(),
                    })?,
                },
                None => Default::default(),
            },
        })
    }
}

fn optional_bool(v: Option<&Value>) -> Option<bool> {
    match v {
        None => Some(false),
        Some(v) => v.as_bool(),
    }
}
