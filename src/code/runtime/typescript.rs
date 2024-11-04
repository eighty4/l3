use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

#[derive(Clone, Default)]
pub struct TypeScriptConfig {
    // https://www.typescriptlang.org/tsconfig/#paths
    // https://www.typescriptlang.org/docs/handbook/modules/reference.html#paths
    #[allow(unused)]
    pub path_aliases: HashMap<String, Vec<PathBuf>>,
}

impl TypeScriptConfig {}

pub fn read_typescript_config(project_dir: &Path) -> TypeScriptConfig {
    let maybe_tsconfig_json: Option<Value> =
        match fs::read_to_string(project_dir.join("tsconfig.json")) {
            Ok(s) => match serde_json::from_str(s.as_str()) {
                Ok(v) => v,
                Err(err) => panic!("error parsing tsconfig.json: {}", err),
            },
            Err(_) => None,
        };
    match maybe_tsconfig_json {
        None => Default::default(),
        Some(tsconfig_json) => TypeScriptConfig {
            path_aliases: read_path_aliases(&tsconfig_json),
        },
    }
}

fn read_path_aliases(tsconfig_json: &Value) -> HashMap<String, Vec<PathBuf>> {
    let mut result = HashMap::new();
    if let Value::Object(compiler_options) = &tsconfig_json["compilerOptions"] {
        if let Value::Object(paths_obj) = &compiler_options["paths"] {
            for (alias, paths_arr) in paths_obj {
                let mut paths = Vec::new();
                if let Value::Array(paths_arr) = paths_arr {
                    for path in paths_arr {
                        if let Value::String(path) = path {
                            paths.push(PathBuf::from(path));
                        }
                    }
                }
                result.insert(alias.clone(), paths);
            }
        }
    }
    result
}
