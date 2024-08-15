use crate::code::runtime::node::NodePackageManager::*;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Default)]
pub enum NodePackageManager {
    #[default]
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
            "node-addons" => NodeConditionalImportKind::NodeAddons,
            "node" => NodeConditionalImportKind::Node,
            "import" => NodeConditionalImportKind::Import,
            "require" => NodeConditionalImportKind::Require,
            "default" => NodeConditionalImportKind::Default,
            &_ => NodeConditionalImportKind::Unknown,
        }
    }
}

#[derive(Clone, Default)]
pub struct NodeConfig {
    #[allow(unused)]
    pub dependencies: Vec<String>,
    #[allow(unused)]
    pub package_manager: NodePackageManager,
    #[allow(unused)]
    pub subpath_imports: HashMap<String, Vec<NodeConditionalImport>>,
}

impl NodeConfig {}

pub fn read_node_config(project_dir: &Path) -> NodeConfig {
    let maybe_package_json: Option<Value> =
        match fs::read_to_string(project_dir.join("package.json")) {
            Ok(s) => match serde_json::from_str(s.as_str()) {
                Ok(v) => v,
                Err(err) => panic!("error parsing package.json: {}", err),
            },
            Err(_) => None,
        };
    let package_manager = read_package_manager(project_dir);
    match maybe_package_json {
        None => NodeConfig {
            package_manager,
            ..Default::default()
        },
        Some(package_json) => NodeConfig {
            dependencies: read_dependencies(&package_json),
            package_manager,
            subpath_imports: read_subpath_imports(&package_json),
        },
    }
}

fn read_dependencies(package_json: &Value) -> Vec<String> {
    let mut dependencies: Vec<String> = Vec::new();
    for dep_key in &["dependencies", "devDependencies"] {
        if let Value::Object(deps) = &package_json[dep_key] {
            for dep in deps.keys() {
                dependencies.push(dep.clone());
            }
        }
    }
    dependencies
}

fn read_package_manager(project_dir: &Path) -> NodePackageManager {
    if project_dir.join("pnpm-lock.yaml").is_file() {
        Pnpm
    } else {
        Npm
    }
}

fn read_subpath_imports(package_json: &Value) -> HashMap<String, Vec<NodeConditionalImport>> {
    let mut result = HashMap::new();
    if let Value::Object(imports) = &package_json["imports"] {
        for (alias, path) in imports {
            result.insert(
                alias.clone(),
                match path {
                    Value::String(s) => vec![NodeConditionalImport {
                        kind: NodeConditionalImportKind::Default,
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
    result
}
