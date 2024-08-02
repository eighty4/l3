use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::code::runtime::javascript::NodeConditionalImportKind::*;
use crate::code::runtime::javascript::NodePackageManager::*;

#[derive(Clone)]
pub enum NodePackageManager {
    Npm,
    Pnpm,
}

#[derive(Clone)]
pub struct NodeConditionalImport {
    #[allow(unused)]
    pub kind: NodeConditionalImportKind,
    #[allow(unused)]
    pub path: PathBuf,
}

// https://nodejs.org/api/packages.html#conditional-exports
#[derive(Clone)]
pub enum NodeConditionalImportKind {
    NodeAddons,
    Node,
    Import,
    Require,
    Default,
    Unknown,
}

impl From<&str> for NodeConditionalImportKind {
    fn from(s: &str) -> Self {
        match s {
            "node-addons" => NodeAddons,
            "node" => Node,
            "import" => Import,
            "require" => Require,
            "default" => Default,
            &_ => Unknown,
        }
    }
}

#[derive(Clone)]
pub struct JavaScriptDeets {
    #[allow(unused)]
    pub dependencies: Vec<String>,
    #[allow(unused)]
    pub package_manager: NodePackageManager,
    #[allow(unused)]
    pub subpath_imports: HashMap<String, Vec<NodeConditionalImport>>,
}

impl JavaScriptDeets {
    pub fn read_details(project_dir: &Path) -> Result<Self, anyhow::Error> {
        Ok(Self {
            dependencies: Self::read_dependencies(project_dir)?,
            package_manager: Self::read_package_manager(project_dir),
            subpath_imports: Self::read_subpath_imports(project_dir)?,
        })
    }

    pub fn has_dependency(&self, dependency: &String) -> bool {
        self.dependencies.contains(dependency)
    }

    fn read_dependencies(project_dir: &Path) -> Result<Vec<String>, anyhow::Error> {
        let package: Value = match fs::read_to_string(project_dir.join("package.json")) {
            Ok(s) => serde_json::from_str(s.as_str())?,
            Err(_) => return Ok(Vec::new()),
        };
        let mut result = Vec::new();
        for dep_key in &["dependencies", "devDependencies"] {
            if let Value::Object(deps) = &package[dep_key] {
                for dep in deps.keys() {
                    result.push(dep.clone());
                }
            }
        }
        Ok(result)
    }

    fn read_package_manager(project_dir: &Path) -> NodePackageManager {
        if project_dir.join("pnpm-lock.yaml").is_file() {
            Pnpm
        } else {
            Npm
        }
    }

    fn read_subpath_imports(
        project_dir: &Path,
    ) -> Result<HashMap<String, Vec<NodeConditionalImport>>, anyhow::Error> {
        let package: Value = match fs::read_to_string(project_dir.join("package.json")) {
            Ok(s) => serde_json::from_str(s.as_str())?,
            Err(_) => return Ok(HashMap::new()),
        };
        let mut result = HashMap::new();
        if let Value::Object(imports) = &package["imports"] {
            for (alias, path) in imports {
                result.insert(
                    alias.clone(),
                    match path {
                        Value::String(s) => vec![NodeConditionalImport {
                            kind: Default,
                            path: PathBuf::from(s),
                        }],
                        Value::Object(paths) => {
                            let mut imports = Vec::new();
                            for (kind, path) in paths {
                                if let Value::String(path) = path {
                                    imports.push(NodeConditionalImport {
                                        kind: NodeConditionalImportKind::from(kind.as_ref()),
                                        path: PathBuf::from(path),
                                    });
                                }
                            }
                            imports
                        }
                        _ => vec![],
                    },
                );
            }
        }
        Ok(result)
    }
}

impl std::default::Default for JavaScriptDeets {
    fn default() -> Self {
        Self {
            dependencies: Vec::new(),
            package_manager: Npm,
            subpath_imports: HashMap::new(),
        }
    }
}
