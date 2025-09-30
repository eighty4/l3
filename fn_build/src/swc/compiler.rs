use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use swc::config::{Config, IsModule, JscConfig, Options};
use swc::{BoolConfig, Compiler};
use swc_common::errors::{Diagnostic, DiagnosticBuilder, Emitter, Handler, HANDLER};
use swc_common::{SourceFile, SourceMap, GLOBALS};
use swc_ecma_ast::{EsVersion, Module, Program};
use swc_ecma_parser::{EsSyntax, Syntax, TsSyntax};

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

    pub fn minify_js(self, path: PathBuf, js: String) -> CompileResult<String> {
        self.string_source_with_compiler(path, js, |compiler, handler, source_file| {
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

    pub fn parse_module(self, path: &Path) -> CompileResult<Module> {
        debug_assert!(path.is_absolute());
        debug_assert!(path.is_file(), "{} does not exist", path.to_string_lossy());
        self.fs_source_with_compiler(path, |compiler, handler, source_file| {
            compiler
                .parse_js(
                    source_file,
                    handler,
                    EsVersion::EsNext,
                    path.extension()
                        .map(|ext| match ext.to_str().unwrap() {
                            "ts" => Syntax::Typescript(TsSyntax::default()),
                            _ => Syntax::Es(EsSyntax::default()),
                        })
                        .unwrap(),
                    IsModule::Bool(true),
                    None,
                )
                .map(|program| match program {
                    Program::Module(module) => module,
                    Program::Script(_) => panic!("cjs"),
                })
        })
    }

    pub fn transpile_ts(self, path: PathBuf, js: String) -> CompileResult<String> {
        self.process_ts(path, js, false)
    }

    pub fn transpile_and_minify_ts(self, path: PathBuf, js: String) -> CompileResult<String> {
        self.process_ts(path, js, true)
    }

    fn process_ts(self, path: PathBuf, js: String, minify: bool) -> CompileResult<String> {
        self.string_source_with_compiler(path, js, |compiler, handler, source_file| {
            compiler
                .process_js_file(
                    source_file,
                    handler,
                    &Options {
                        config: Config {
                            jsc: JscConfig {
                                syntax: Some(Syntax::Typescript(TsSyntax {
                                    decorators: false,
                                    disallow_ambiguous_jsx_like: true,
                                    tsx: false,
                                    dts: false,
                                    no_early_errors: false,
                                })),
                                target: Some(EsVersion::Es2024),
                                ..Default::default()
                            },
                            is_module: Some(IsModule::Bool(true)),
                            minify: BoolConfig::new(Some(minify)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )
                .map(|transform_output| transform_output.code)
        })
    }

    fn fs_source_with_compiler<F, R>(self, p: &Path, f: F) -> CompileResult<R>
    where
        F: FnOnce(&Compiler, &Handler, Arc<SourceFile>) -> Result<R, anyhow::Error>,
    {
        let source_file = self.source_map.load_file(p)?;
        self.with_compiler(|compiler, handler| f(compiler, handler, source_file))
    }

    fn string_source_with_compiler<F, R>(self, p: PathBuf, js: String, f: F) -> CompileResult<R>
    where
        F: FnOnce(&Compiler, &Handler, Arc<SourceFile>) -> Result<R, anyhow::Error>,
    {
        let source_file = self.source_map.new_source_file(Arc::new(p.into()), js);
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
