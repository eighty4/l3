use crate::runtime::node::imports::subpath::{
    NodeSubpathImportMapping, NodeSubpathImports, SubpathImportError,
};
use serde_json::{Map, Value};

/// Defined at <https://nodejs.org/api/packages.html#conditional-exports>.
enum NodeSubpathImportCondition {
    NodeAddons,
    Node,
    Import,
    Require,
    ModuleSync,
    Default,
    Unknown(String),
}

/// Creates a NodeSubpathImports from JSON map of package.json's imports object.
///
/// Surfaces SubpathImportError for when NodeSubpathImportCondition::Unknown encountered to be
/// consistent with Node runtime error handling when package.json import conditions have unsupported
/// condition keys.
///
/// Read docs at <https://nodejs.org/api/packages.html#subpath-imports>.
pub fn read_subpath_imports(
    imports: &Map<String, Value>,
) -> Result<NodeSubpathImports, SubpathImportError> {
    let mut result: NodeSubpathImports = Vec::new();
    for (map_from, map_to) in imports {
        if map_from.starts_with('#') {
            if let Some(map_to) = map_to.as_str() {
                result.push(NodeSubpathImportMapping::new(
                    map_from.clone(),
                    map_to.to_string(),
                ));
            } else if let Some(map_to) = map_to.as_object() {
                let maybe_mapping = find_valid_subpath_import_from_conditionals(map_from, map_to)?;
                if let Some(mapping) = maybe_mapping {
                    result.push(mapping);
                }
            }
        }
    }
    Ok(result)
}

fn find_valid_subpath_import_from_conditionals(
    map_from: &String,
    conditions_json: &Map<String, Value>,
) -> Result<Option<NodeSubpathImportMapping>, SubpathImportError> {
    for (condition, map_to) in conditions_json {
        let condition = NodeSubpathImportCondition::from(condition.as_str());
        if let NodeSubpathImportCondition::Unknown(unknown) = condition {
            return Err(SubpathImportError::BadImportCondition(unknown));
        } else if condition.is_es_module_compatible() {
            if let Some(map_to) = map_to.as_str() {
                return Ok(Some(NodeSubpathImportMapping::new(
                    map_from.clone(),
                    map_to.to_string(),
                )));
            } else if let Some(map_to) = map_to.as_object() {
                match find_valid_subpath_import_from_conditionals(map_from, map_to)? {
                    None => continue,
                    Some(mapping) => return Ok(Some(mapping)),
                }
            }
        }
    }
    Ok(None)
}

impl NodeSubpathImportCondition {
    pub fn is_es_module_compatible(&self) -> bool {
        matches!(
            self,
            Self::NodeAddons | Self::Node | Self::Import | Self::Default
        )
    }
}

impl From<&str> for NodeSubpathImportCondition {
    fn from(condition: &str) -> Self {
        match condition.to_lowercase().as_str() {
            "node-addons" => Self::NodeAddons,
            "node" => Self::Node,
            "import" => Self::Import,
            "require" => Self::Require,
            "module-sync" => Self::ModuleSync,
            "default" => Self::Default,
            _ => Self::Unknown(condition.to_string()),
        }
    }
}
