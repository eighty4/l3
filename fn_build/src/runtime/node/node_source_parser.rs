use crate::runtime::node::{NodeConfig, NodeImportResolver};
use crate::runtime::{FnSourceParser, ImportResolver};
use crate::swc::compiler::SwcCompiler;
use crate::swc::visitors::ImportVisitor;
use crate::{FnBuildResult, FnSource};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use swc_ecma_visit::FoldWith;

pub struct NodeFnSourceParser {
    compiler: SwcCompiler,
    import_resolver: Arc<NodeImportResolver>,
}

impl NodeFnSourceParser {
    pub fn new(node_config: Arc<NodeConfig>) -> Self {
        Self {
            compiler: SwcCompiler::new(),
            import_resolver: Arc::new(NodeImportResolver::new(node_config)),
        }
    }
}

impl FnSourceParser for NodeFnSourceParser {
    fn collect_runtime_sources(&self, project_dir: &Path) -> Vec<FnSource> {
        let package_json_path = PathBuf::from("package.json");
        if project_dir.join(&package_json_path).is_file() {
            vec![FnSource::from(package_json_path)]
        } else {
            Vec::new()
        }
    }

    fn parse(&self, project_dir: &Path, source_path: PathBuf) -> FnBuildResult<FnSource> {
        debug_assert!(project_dir.is_absolute());
        debug_assert!(project_dir.is_dir());
        debug_assert!(source_path.is_relative());
        let module = self
            .compiler
            .clone()
            .parse_es_module(&project_dir.join(&source_path))
            .unwrap();
        let mut visitor = ImportVisitor::new();
        module.fold_with(&mut visitor);
        let imports = visitor
            .result()
            .into_iter()
            .map(|import| {
                self.import_resolver
                    .resolve(project_dir, &source_path, import.as_str())
            })
            .collect();
        Ok(FnSource {
            imports,
            path: source_path.to_path_buf(),
        })
    }
}
