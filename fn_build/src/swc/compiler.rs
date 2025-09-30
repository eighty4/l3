use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};
use swc::config::IsModule;
use swc::Compiler;
use swc_common::errors::{Diagnostic, DiagnosticBuilder, Emitter, Handler, HANDLER};
use swc_common::{SourceFile, SourceMap, GLOBALS};
use swc_ecma_ast::{EsVersion, Module, Program};
use swc_ecma_parser::{EsSyntax, Syntax};

#[derive(Clone)]
struct CapturingEmitter {
    errors: Arc<Mutex<Vec<Diagnostic>>>,
}

impl CapturingEmitter {
    pub fn new(errors: Arc<Mutex<Vec<Diagnostic>>>) -> Self {
        Self { errors }
    }
}

impl Emitter for CapturingEmitter {
    fn emit(&mut self, db: &mut DiagnosticBuilder<'_>) {
        self.errors.lock().unwrap().push((**db).clone());
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CompileError {
    #[error("compiler produced diagnostic info")]
    CompilerDiagnostics(Vec<Diagnostic>),
    #[error("compiler operation produced error: {0}")]
    OperationError(String),
    #[error("reading source io error: {0}")]
    ReadError(#[from] io::Error),
}

pub type CompileResult<R> = Result<R, CompileError>;

#[derive(Clone)]
pub struct SwcCompiler {
    compiler: Arc<Compiler>,
    pub source_map: Arc<SourceMap>,
}

impl SwcCompiler {
    pub fn new() -> Self {
        let source_map = Arc::<SourceMap>::default();
        let compiler = Arc::new(Compiler::new(source_map.clone()));
        Self {
            compiler,
            source_map,
        }
    }

    pub fn minify_js(self, path: &Path) -> CompileResult<String> {
        self.source_with_compiler(path, |compiler, handler, source_file| {
            compiler
                .minify(
                    source_file,
                    handler,
                    &Default::default(),
                    Default::default(),
                )
                .map(|transform_output| transform_output.code)
        })
    }

    pub fn parse_es_module(self, path: &Path) -> CompileResult<Module> {
        debug_assert!(path.is_absolute());
        debug_assert!(path.is_file());
        self.source_with_compiler(path, |compiler, handler, source_file| {
            compiler
                .parse_js(
                    source_file,
                    handler,
                    EsVersion::EsNext,
                    Syntax::Es(EsSyntax::default()),
                    IsModule::Bool(true),
                    None,
                )
                .map(|program| match program {
                    Program::Module(module) => module,
                    Program::Script(_) => panic!("cjs"),
                })
        })
    }

    fn source_with_compiler<F, R>(self, p: &Path, f: F) -> CompileResult<R>
    where
        F: FnOnce(&Compiler, &Handler, Arc<SourceFile>) -> Result<R, anyhow::Error>,
    {
        let source_file = self.source_map.load_file(p)?;
        self.with_compiler(|compiler, handler| f(compiler, handler, source_file))
    }

    fn with_compiler<F, R>(self, f: F) -> CompileResult<R>
    where
        F: FnOnce(&Compiler, &Handler) -> Result<R, anyhow::Error>,
    {
        GLOBALS.set(&Default::default(), || {
            let errors: Arc<Mutex<Vec<Diagnostic>>> = Default::default();
            let result = {
                let emitter = CapturingEmitter::new(errors.clone());
                let handler = Handler::with_emitter(true, false, Box::new(emitter));
                HANDLER.set(&handler, || f(&self.compiler, &handler))
            };
            let diagnostics: Vec<Diagnostic> = Arc::into_inner(errors)
                .unwrap_or_default()
                .into_inner()
                .unwrap_or_default();
            if diagnostics.is_empty() {
                match result {
                    Ok(result) => Ok(result),
                    Err(err) => Err(CompileError::OperationError(err.to_string())),
                }
            } else {
                Err(CompileError::CompilerDiagnostics(diagnostics))
            }
        })
    }
}
