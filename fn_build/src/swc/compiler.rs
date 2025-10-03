use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use swc::config::{Config, IsModule, JscConfig, Options};
use swc::{BoolConfig, Compiler};
use swc_common::errors::{Diagnostic, DiagnosticBuilder, Emitter, Handler, HANDLER};
use swc_common::{SourceFile, SourceMap, GLOBALS};
use swc_ecma_ast::{noop_pass, EsVersion, Module, Pass, Program};
use swc_ecma_parser::{EsSyntax, Syntax, TsSyntax};
use swc_ecma_visit::fold_pass;

use crate::swc::visitors::RewriteTsImportsVisitor;

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
    /**
     * swc::Compiler APIs
     *
     * parse_js: (source_file) => program
     * transform: (program, fold) => program
     * process_js: (program) => string
     * process_js_file: (source_file) => string
     * process_js_with_custom_pass: (source_file, program?, fold) => string
     * minify: (source_file) => string
     */
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

    // from string of js, minify
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

    // from file of ts or js, parse to ast
    pub fn parse_program_from_fs(self, path: &Path) -> CompileResult<Program> {
        debug_assert!(path.is_absolute());
        self.fs_source_with_compiler(path, |compiler, handler, source_file| {
            compiler
                .parse_js(
                    source_file,
                    handler,
                    es_target(),
                    es_or_ts_syntax(path),
                    IsModule::Bool(true),
                    None,
                )
                .map(|program| {
                    if let Program::Script(_) = program {
                        panic!("cjs");
                    }
                    program
                })
        })
    }

    // from file of ts or js, parse to ast
    pub fn parse_module_from_fs(self, path: &Path) -> CompileResult<Module> {
        debug_assert!(path.is_absolute());
        self.parse_program_from_fs(path)
            .map(|program| match program {
                Program::Module(module) => module,
                Program::Script(_) => panic!("cjs"),
            })
    }

    #[allow(dead_code)]
    pub fn transform_to_string_from_ast(self, program: Program) -> CompileResult<String> {
        self.with_compiler(|compiler, handler| {
            Ok(compiler
                .process_js(handler, program, &process_opts(false, ts_syntax()))?
                .code)
        })
    }

    // from string of ts code, transpile to js
    pub fn transpile_ts(
        self,
        path: PathBuf,
        ts: String,
        rewrite_ts_imports: bool,
    ) -> CompileResult<String> {
        self.process_ts(path, ts, false, rewrite_ts_imports)
    }

    // from string of ts code, transpile to js and minify
    pub fn transpile_and_minify_ts(
        self,
        path: PathBuf,
        ts: String,
        rewrite_ts_imports: bool,
    ) -> CompileResult<String> {
        self.process_ts(path, ts, true, rewrite_ts_imports)
    }

    // from string of ts code, transpile to js, optionally minify
    fn process_ts(
        self,
        path: PathBuf,
        ts: String,
        minify: bool,
        rewrite_ts_imports: bool,
    ) -> CompileResult<String> {
        self.string_source_with_compiler(path, ts, |compiler, handler, source_file| {
            let after_pass: Box<dyn Pass> = match rewrite_ts_imports {
                true => Box::new(fold_pass(RewriteTsImportsVisitor::new())),
                false => Box::new(noop_pass()),
            };
            compiler
                .process_js_with_custom_pass(
                    source_file,
                    None,
                    handler,
                    &process_opts(minify, ts_syntax()),
                    Default::default(),
                    |_| noop_pass(),
                    |_| after_pass,
                )
                .map(|transform_output| transform_output.code)
        })
    }
}

fn process_opts(minify: bool, syntax: Syntax) -> Options {
    Options {
        config: Config {
            jsc: JscConfig {
                syntax: Some(syntax),
                target: Some(es_target()),
                ..Default::default()
            },
            is_module: Some(IsModule::Bool(true)),
            minify: BoolConfig::new(Some(minify)),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn es_or_ts_syntax(p: &Path) -> Syntax {
    p.extension()
        .map(|ext| match ext.to_str().unwrap() {
            "ts" => ts_syntax(),
            _ => es_syntax(),
        })
        .unwrap()
}

fn es_syntax() -> Syntax {
    Syntax::Es(EsSyntax::default())
}

fn ts_syntax() -> Syntax {
    Syntax::Typescript(TsSyntax {
        decorators: false,
        disallow_ambiguous_jsx_like: true,
        tsx: false,
        dts: false,
        no_early_errors: false,
    })
}

fn es_target() -> EsVersion {
    EsVersion::EsNext
}

impl SwcCompiler {
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
