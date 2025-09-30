use crate::runtime::node::imports::resolver::NodeImportResolver;
use crate::runtime::node::NodeConfig;
use crate::runtime::parse_fn::parse_fn_inner;
use crate::runtime::ts_imports::TypeScriptImportResolver;
use crate::runtime::{FnSourceParser, ImportResolver, Runtime};
use crate::swc::compiler::{CompileError, SwcCompiler};
use crate::swc::visitors::ImportVisitor;
use crate::{
    FnEntrypoint, FnHandler, FnParseError, FnParseManifest, FnParseResult, FnParseSpec, FnSource,
    ModuleImport,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use swc_ecma_ast::{Decl, ExportDecl, Module, ModuleDecl};
use swc_ecma_visit::FoldWith;

pub async fn parse_node_entrypoint(parse_spec: FnParseSpec) -> FnParseResult<FnEntrypoint> {
    let source_parser = match &parse_spec.runtime {
        Runtime::Node(node_config) => create_parser(node_config, &parse_spec.project_dir)?,
        _ => panic!(),
    };
    Ok(FnEntrypoint {
        handlers: source_parser
            .collect_handlers(&parse_spec.project_dir, &parse_spec.entrypoint)?,
        path: parse_spec.entrypoint,
    })
}

pub async fn parse_node_fn(parse_spec: FnParseSpec) -> FnParseResult<FnParseManifest> {
    parse_fn_inner(
        &parse_spec,
        match &parse_spec.runtime {
            Runtime::Node(node_config) => create_parser(node_config, &parse_spec.project_dir)?,
            _ => panic!(),
        },
    )
    .await
}

/// Initializes NodeConfig from project_dir if not provided and creates a NodeFnSourceParser.
/// NodeFnSourceParser determines whether to use TS+Node or vanilla Node import resolution.
fn create_parser(
    maybe_node_config: &Option<Arc<NodeConfig>>,
    project_dir: &Path,
) -> FnParseResult<Arc<Box<dyn FnSourceParser>>> {
    let node_config = match maybe_node_config {
        Some(nc) => nc.clone(),
        None => Arc::new(NodeConfig::read_configs(project_dir)?),
    };
    Ok(Arc::new(Box::new(NodeFnSourceParser::new(node_config))))
}

impl From<CompileError> for FnParseError {
    fn from(err: CompileError) -> Self {
        match err {
            CompileError::CompilerDiagnostics(diagnostics) => {
                // todo map SWC diagnostics to a public API type
                dbg!(diagnostics);
                FnParseError::SyntaxError
            }
            CompileError::OperationError(err) => todo!("compiler op error: {}", err),
            CompileError::ReadError(err) => FnParseError::IoError(err),
        }
    }
}

struct NodeFnSourceParser {
    compiler: SwcCompiler,
    import_resolver: Arc<Box<dyn ImportResolver>>,
}

impl NodeFnSourceParser {
    fn new(node_config: Arc<NodeConfig>) -> Self {
        Self {
            compiler: SwcCompiler::new(),
            import_resolver: match &node_config.ts {
                Some(tsconfig) => Arc::new(Box::new(TypeScriptImportResolver::new(
                    tsconfig.clone(),
                    Box::new(NodeImportResolver::new(node_config)),
                ))),
                None => Arc::new(Box::new(NodeImportResolver::new(node_config))),
            },
        }
    }

    fn parse_module(&self, project_dir: &Path, source_path: &Path) -> FnParseResult<Module> {
        Ok(self
            .compiler
            .clone()
            .parse_module(&project_dir.join(source_path))?)
    }

    fn collect_imports(
        &self,
        project_dir: &Path,
        source_path: &Path,
    ) -> FnParseResult<Vec<ModuleImport>> {
        let module = self.parse_module(project_dir, source_path)?;
        let mut visitor = ImportVisitor::new();
        module.fold_with(&mut visitor);
        let imports = visitor
            .result()
            .into_iter()
            .map(|import| {
                self.import_resolver
                    .resolve(project_dir, source_path, import.as_str())
            })
            .collect();
        Ok(imports)
    }
}

impl FnSourceParser for NodeFnSourceParser {
    fn collect_handlers(
        &self,
        project_dir: &Path,
        source_path: &Path,
    ) -> FnParseResult<Vec<FnHandler>> {
        let module = self.parse_module(project_dir, source_path)?;
        let mut handlers: Vec<FnHandler> = Vec::new();
        let parse_fn_name = |s: &str| {
            s.trim_end_matches(char::is_numeric)
                .trim_end_matches(char::is_numeric)
                .trim_end_matches('#')
                .to_string()
        };
        let create_handler = |s: &str| FnHandler::from_handler_fn(source_path, parse_fn_name(s));
        for module_item in module.body {
            if let Some(module_decl) = module_item.module_decl() {
                match module_decl {
                    ModuleDecl::ExportDecl(ExportDecl {
                        decl: Decl::Fn(fn_decl),
                        ..
                    }) => handlers.push(create_handler(fn_decl.ident.as_ref())),
                    ModuleDecl::ExportDecl(ExportDecl {
                        decl: Decl::Var(var_decl),
                        ..
                    }) => {
                        for var_declarator in var_decl.decls {
                            if let Some(expr) = var_declarator.init {
                                if expr.as_arrow().is_some() || expr.as_fn_expr().is_some() {
                                    handlers.push(create_handler(
                                        var_declarator.name.ident().unwrap().as_ref(),
                                    ))
                                }
                            }
                        }
                    }
                    _ => {}
                }
            };
        }
        Ok(handlers)
    }

    fn collect_runtime_sources(&self, project_dir: &Path) -> Vec<FnSource> {
        let package_json_path = PathBuf::from("package.json");
        if project_dir.join(&package_json_path).is_file() {
            vec![FnSource {
                imports: Vec::new(),
                path: package_json_path,
            }]
        } else {
            Vec::new()
        }
    }

    fn parse_fn_entrypoint(
        &self,
        project_dir: &Path,
        source_path: PathBuf,
    ) -> FnParseResult<(FnSource, Vec<FnHandler>)> {
        let handlers = self.collect_handlers(project_dir, &source_path)?;
        let source = self.parse_for_imports(project_dir, source_path)?;
        Ok((source, handlers))
    }

    fn parse_for_imports(
        &self,
        project_dir: &Path,
        source_path: PathBuf,
    ) -> FnParseResult<FnSource> {
        debug_assert!(project_dir.is_absolute());
        debug_assert!(project_dir.is_dir());
        debug_assert!(source_path.is_relative());
        Ok(FnSource {
            imports: self.collect_imports(project_dir, &source_path)?,
            path: source_path.to_path_buf(),
        })
    }
}
