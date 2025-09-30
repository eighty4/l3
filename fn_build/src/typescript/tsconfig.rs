use std::{fs::read_to_string, path::Path, sync::Arc};

use serde_json::Value;

use crate::runtime::node::NodeConfigError;

#[allow(unused)]
#[derive(Debug, Default)]
pub struct TsConfigJson {
    compiler: TsCompilerOptions,
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct TsCompilerOptions {
    /// Captures compilerOptions.allowImportingTsExtensions.
    allow_importing_ts: bool,
    /// Captures compilerOptions.allowJs.
    allow_js: bool,
    /// Captures compilerOptions.rewriteRelativeImportExtensions.
    rewrite_relative_imports: bool,
}

impl TsConfigJson {
    fn parse_tsconfig_json(tsconfig_json: &str) -> Result<Arc<Self>, NodeConfigError> {
        Ok(Arc::new(Self::try_from(serde_json::from_str::<Value>(
            tsconfig_json,
        )?)?))
    }

    pub fn read_tsconfig_json(path: &Path) -> Result<Arc<Self>, NodeConfigError> {
        debug_assert!(path.extension().unwrap() == "json");
        if path.is_file() {
            Self::parse_tsconfig_json(read_to_string(path)?.as_str())
        } else {
            Ok(Default::default())
        }
    }
}

impl TryFrom<Value> for TsConfigJson {
    type Error = NodeConfigError;

    fn try_from(tsconfig_json: Value) -> Result<Self, Self::Error> {
        Ok(Self {
            compiler: match tsconfig_json
                .get("compilerOptions")
                .and_then(|co| co.as_object())
            {
                Some(co_json) => TsCompilerOptions {
                    allow_importing_ts: co_json
                        .get("allowImportingTsExtensions")
                        .and_then(|v| v.as_bool())
                        .unwrap_or_default(),
                    allow_js: co_json
                        .get("allowJs")
                        .and_then(|v| v.as_bool())
                        .unwrap_or_default(),
                    rewrite_relative_imports: co_json
                        .get("rewriteRelativeImportExtensions")
                        .and_then(|v| v.as_bool())
                        .unwrap_or_default(),
                },
                None => Default::default(),
            },
        })
    }
}
