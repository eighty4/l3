use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

#[derive(Clone, Default)]
pub struct TypeScriptDeets {
    // https://www.typescriptlang.org/tsconfig/#paths
    // https://www.typescriptlang.org/docs/handbook/modules/reference.html#paths
    #[allow(unused)]
    ts_path_aliases: HashMap<String, Vec<PathBuf>>,
}

impl TypeScriptDeets {
    pub fn read_details(project_dir: &Path) -> Result<Self, anyhow::Error> {
        Ok(if project_dir.join("tsconfig.json").is_file() {
            Self {
                ts_path_aliases: Self::read_ts_path_aliases(project_dir)?,
            }
        } else {
            Default::default()
        })
    }

    fn read_ts_path_aliases(
        project_dir: &Path,
    ) -> Result<HashMap<String, Vec<PathBuf>>, anyhow::Error> {
        let tsconfig: Value = match fs::read_to_string(project_dir.join("tsconfig.json")) {
            Ok(s) => serde_json::from_str(s.as_str())?,
            Err(_) => return Ok(HashMap::new()),
        };
        let mut result = HashMap::new();
        if let Value::Object(compiler_options) = &tsconfig["compilerOptions"] {
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
        Ok(result)
    }
}
