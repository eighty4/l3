use crate::paths::join_file_paths;
use crate::runtime::parse_fn::parse_fn_inner;
use crate::runtime::{FnEntrypoint, FnSourceParser, ImportResolver};
use crate::{
    FnHandler, FnParseError, FnParseManifest, FnParseResult, FnParseSpec, FnSource, ModuleImport,
};
use rustpython_parser::ast::Stmt;
use rustpython_parser::{ast, Parse, ParseError};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub async fn parse_python_entrypoint(parse_spec: FnParseSpec) -> FnParseResult<Vec<FnHandler>> {
    let ast = PythonSourceParser::parse_ast(&parse_spec.project_dir, &parse_spec.entrypoint)?;
    let handlers = PythonSourceParser::collect_handlers(&parse_spec.entrypoint, &ast);
    Ok(handlers)
}

pub async fn parse_python_fn(parse_spec: FnParseSpec) -> FnParseResult<FnParseManifest> {
    parse_fn_inner(&parse_spec, Arc::new(Box::new(PythonSourceParser::new()))).await
}

impl From<ParseError> for FnParseError {
    fn from(err: ParseError) -> Self {
        // todo map rustpython_parser::ParseError diagnostics to a public API type
        dbg!(err);
        FnParseError::SyntaxError
    }
}

struct PythonSourceParser {
    import_resolver: PythonImportResolver,
}

impl PythonSourceParser {
    fn new() -> Self {
        Self {
            import_resolver: PythonImportResolver {},
        }
    }

    fn parse_ast(project_dir: &Path, path: &Path) -> FnParseResult<Vec<Stmt>> {
        let abs_path = project_dir.join(path);
        let python_code = fs::read_to_string(abs_path)?;
        let ast = ast::Suite::parse(&python_code, &path.to_string_lossy())?;
        Ok(ast)
    }

    fn collect_handlers(source_path: &Path, ast: &Vec<Stmt>) -> Vec<FnHandler> {
        let mut handlers: Vec<FnHandler> = Vec::new();
        for stmt in ast {
            match stmt {
                Stmt::FunctionDef(function) => handlers.push(FnHandler::from_handler_fn(
                    source_path,
                    function.name.to_string(),
                )),
                Stmt::AsyncFunctionDef(_) => todo!("to support python async functions as handlers, build_python_fn will have to generate code for a non async handler that launches teh async python function with the python async runtime"),
                _ => {}
            }
        }
        handlers
    }

    fn collect_imports(
        &self,
        project_dir: &Path,
        source_path: &Path,
        ast: &Vec<Stmt>,
    ) -> Vec<ModuleImport> {
        let mut import_specifiers: Vec<String> = Vec::new();
        for stmt in ast {
            match stmt {
                Stmt::Import(import) => {
                    import_specifiers.push(import.names.first().cloned().unwrap().name.to_string())
                }
                Stmt::ImportFrom(_import_from) => {}
                _ => {}
            }
        }
        import_specifiers
            .iter()
            .map(|specifier| {
                self.import_resolver
                    .resolve(project_dir, source_path, specifier.as_str())
            })
            .collect()
    }
}

impl FnSourceParser for PythonSourceParser {
    fn collect_runtime_sources(&self, _project_dir: &Path) -> Vec<FnSource> {
        Vec::new()
    }

    fn parse_fn_entrypoint(
        &self,
        project_dir: &Path,
        path: PathBuf,
    ) -> FnParseResult<FnEntrypoint> {
        let ast = Self::parse_ast(project_dir, &path)?;
        let handlers = Self::collect_handlers(&path, &ast);
        let imports = self.collect_imports(project_dir, &path, &ast);
        Ok(FnEntrypoint {
            handlers,
            source: FnSource { imports, path },
        })
    }

    fn parse_for_imports(&self, project_dir: &Path, path: PathBuf) -> FnParseResult<FnSource> {
        let ast = Self::parse_ast(project_dir, &path)?;
        let imports = self.collect_imports(project_dir, &path, &ast);
        Ok(FnSource { imports, path })
    }
}

struct PythonImportResolver {}

impl ImportResolver for PythonImportResolver {
    fn resolve(&self, project_dir: &Path, from: &Path, import: &str) -> ModuleImport {
        dbg!(project_dir);
        dbg!(from);
        dbg!(import);
        ModuleImport::RelativeSource(join_file_paths(
            from,
            &PathBuf::from(format!("./{import}.py")),
        ))
    }
}
