use crate::paths::join_file_paths;
use crate::result::ModuleImport;
use crate::runtime::node::imports::{
    NodeSubpathImportAsterisks, NodeSubpathImportMapWildcardTo, NodeSubpathImportMapping,
};
use crate::runtime::node::NodeConfig;
use crate::runtime::ImportResolver;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// todo nodejs subpath imports
//  https://nodejs.org/api/packages.html#subpath-imports
// todo cross-check swc implementation
//  https://github.com/swc-project/swc/blob/main/crates/swc_ecma_loader/src/resolvers/node.rs
pub struct NodeImportResolver {
    pub node_config: Arc<NodeConfig>,
    package_json: PathBuf,
}

impl NodeImportResolver {
    pub fn new(node_config: Arc<NodeConfig>) -> NodeImportResolver {
        Self {
            node_config,
            package_json: PathBuf::from("package.json"),
        }
    }

    fn resolve_relative_path(
        &self,
        project_dir: &Path,
        from: &Path,
        import: &str,
    ) -> Option<PathBuf> {
        debug_assert!(import.starts_with('.'));
        let maybe = join_file_paths(from, &PathBuf::from(import));
        if project_dir.join(&maybe).is_file() {
            Some(maybe)
        } else {
            None
        }
    }

    fn resolve_subpath_import(&self, project_dir: &Path, import: &str) -> Option<ModuleImport> {
        for mapping in &self.node_config.subpath_imports {
            match &mapping {
                NodeSubpathImportMapping::Explicit { from, to } => {
                    if import == from {
                        return self.resolve_subpath_import_mapped_explicitly(project_dir, to);
                    }
                }
                NodeSubpathImportMapping::Wildcard { from, to } => {
                    return match import.strip_prefix(&from.before) {
                        None => continue,
                        Some(import_remainder) => match to {
                            NodeSubpathImportMapWildcardTo::Explicit(to) => self
                                .resolve_subpath_import_mapping_wildcard_to_explicit_specifier(
                                    project_dir,
                                    to,
                                ),
                            NodeSubpathImportMapWildcardTo::Wildcard(to, asterisks) => self
                                .resolve_subpath_import_mapping_wildcard_to_wildcard(
                                    project_dir,
                                    from.after
                                        .as_ref()
                                        .and_then(|from_after_asterisk| {
                                            import_remainder.strip_suffix(from_after_asterisk)
                                        })
                                        .unwrap_or(import_remainder),
                                    to,
                                    asterisks,
                                ),
                        },
                    }
                }
            }
        }
        None
    }

    fn resolve_subpath_import_mapped_explicitly(
        &self,
        project_dir: &Path,
        to: &str,
    ) -> Option<ModuleImport> {
        if to.starts_with('.') {
            self.resolve_relative_path(project_dir, &self.package_json, to)
                .map(ModuleImport::RelativeSource)
        } else {
            Some(Self::create_package_dependency(to.to_string()))
        }
    }

    fn resolve_subpath_import_mapping_wildcard_to_wildcard(
        &self,
        project_dir: &Path,
        substitution: &str,
        to: &str,
        asterisks: &NodeSubpathImportAsterisks,
    ) -> Option<ModuleImport> {
        let to = match asterisks {
            NodeSubpathImportAsterisks::Single => to.replacen('*', substitution, 1),
            NodeSubpathImportAsterisks::Multiple => to.replace('*', substitution),
        };
        if to.starts_with('.') {
            self.resolve_relative_path(project_dir, &self.package_json, to.as_str())
                .map(ModuleImport::RelativeSource)
        } else {
            Some(Self::create_package_dependency(to))
        }
    }

    fn resolve_subpath_import_mapping_wildcard_to_explicit_specifier(
        &self,
        project_dir: &Path,
        to: &String,
    ) -> Option<ModuleImport> {
        if to.starts_with('.') {
            self.resolve_relative_path(project_dir, &self.package_json, to.as_str())
                .map(ModuleImport::RelativeSource)
        } else {
            Some(Self::create_package_dependency(to.to_string()))
        }
    }

    fn create_package_dependency(specifier: String) -> ModuleImport {
        let (package, subpath) = match specifier.split_once('/') {
            None => (specifier, None),
            Some((before, after)) => (before.to_string(), Some(after.to_string())),
        };
        ModuleImport::PackageDependency { package, subpath }
    }
}

impl ImportResolver for NodeImportResolver {
    fn resolve(&self, project_dir: &Path, from: &Path, import: &str) -> ModuleImport {
        if import.starts_with('.') {
            if let Some(relative_path) = self.resolve_relative_path(project_dir, from, import) {
                return ModuleImport::RelativeSource(relative_path);
            }
        } else if import.starts_with('#') {
            if let Some(subpath_import) = self.resolve_subpath_import(project_dir, import) {
                return subpath_import;
            }
        }
        ModuleImport::Unknown(import.to_string())
    }
}
