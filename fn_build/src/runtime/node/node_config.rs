use crate::runtime::node::imports::{read_subpath_imports, NodeSubpathImports, SubpathImportError};
use crate::typescript::{TsConfigError, TsConfigJson};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use std::{fs, io};

/// Errors that occur when building a NodeConfig from package.json
/// and tsconfig.json.
#[derive(Debug, thiserror::Error)]
#[error("error initializing package.json: {0}")]
pub enum NodeConfigError {
    IoRead(#[from] io::Error),
    JsonParse(#[from] serde_json::Error),
    SubpathImport(#[from] SubpathImportError),
    TsConfig(#[from] TsConfigError),
}

pub struct NodeConfig {
    pub package: PackageJson,
    pub ts: Option<Arc<TsConfigJson>>,
    // version: Option<NodeVersion>
}

/// Parsed data from package.json necessary for building functions for Node.js.
#[derive(Default)]
pub struct PackageJson {
    /// Package names from package.json's "dependencies" object.
    dependencies: Vec<String>,
    /// True if package.json's "type" is explicitly set to "module".
    module_type: bool,
    pub subpath_imports: NodeSubpathImports,
}

impl NodeConfig {
    pub fn read_configs(project_dir: &Path) -> Result<Self, NodeConfigError> {
        let package_json_p = project_dir.join("package.json");
        let ts_config_p = project_dir.join("tsconfig.json");

        Ok(Self {
            package: if package_json_p.exists() {
                PackageJson::read_package_json(&package_json_p)?
            } else {
                Default::default()
            },
            ts: if ts_config_p.is_file() {
                Some(TsConfigJson::read_tsconfig_json(&ts_config_p)?)
            } else {
                None
            },
        })
    }
}

impl PackageJson {
    fn parse_package_json(package_json: &str) -> Result<Self, NodeConfigError> {
        Self::try_from(serde_json::from_str::<Value>(package_json)?)
    }

    pub fn read_package_json(path: &Path) -> Result<Self, NodeConfigError> {
        let path = if path
            .file_name()
            .map(|filename| filename == "package.json")
            .unwrap_or(false)
        {
            path.to_path_buf()
        } else {
            path.join("package.json")
        };
        if path.is_file() {
            Self::parse_package_json(fs::read_to_string(path)?.as_str())
        } else {
            Ok(Default::default())
        }
    }

    pub fn has_npm_dependency(&self, package: &String) -> bool {
        self.dependencies.contains(package)
    }

    pub fn is_es_module(&self) -> bool {
        self.module_type
    }
}

impl TryFrom<Value> for PackageJson {
    type Error = NodeConfigError;

    fn try_from(package_json: Value) -> Result<Self, Self::Error> {
        Ok(Self {
            dependencies: match &package_json.get("dependencies") {
                None => Vec::new(),
                Some(dependencies) => match dependencies.as_object() {
                    None => Vec::new(),
                    Some(dependencies) => dependencies.keys().cloned().collect(),
                },
            },
            module_type: package_json["type"]
                .as_str()
                .map(|t| t == "module")
                .unwrap_or(false),
            subpath_imports: match &package_json.get("imports") {
                None => Vec::new(),
                Some(imports) => match imports.as_object() {
                    None => Vec::new(),
                    Some(imports) => read_subpath_imports(imports)?,
                },
            },
        })
    }
}
