pub mod visitors;

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use swc::config::{Config, IsModule, JscConfig, Options};
use swc::{BoolConfig, Compiler};
use swc_common::errors::{Diagnostic, DiagnosticBuilder, Emitter, Handler, HANDLER};
use swc_common::{SourceFile, SourceMap, GLOBALS};
use swc_ecma_ast::{EsVersion, Program};
use swc_ecma_parser::{EsSyntax, Syntax, TsSyntax};

pub type CompileResult<R> = Result<R, CompileError>;

#[derive(thiserror::Error, Debug)]
pub enum CompileError {
    #[error("compiler produced diagnostic info")]
    CompilerDiagnostics(Vec<Diagnostic>),
    #[error("compiler operation produced error: {0}")]
    OperationError(String),
    #[error("reading source io error: {0}")]
    ReadError(#[from] io::Error),
}

pub fn parse_program_from_fs(path: &Path, compiler: Option<SwcCompiler>) -> CompileResult<Program> {
    let program = compiler.unwrap_or_default().fs_source_with_compiler(
        path,
        |compiler, handler, source_file| {
            compiler.parse_js(
                source_file,
                handler,
                es_target(),
                es_or_ts_syntax(path),
                IsModule::Bool(true),
                None,
            )
        },
    )?;
    if let Program::Script(_) = program {
        panic!("cjs unsupported");
    }
    Ok(program)
}

pub fn parse_program_to_string(
    program: Program,
    compiler: Option<SwcCompiler>,
) -> CompileResult<String> {
    compiler
        .unwrap_or_default()
        .with_compiler(|compiler, handler| {
            Ok(compiler
                .process_js(handler, program, &process_opts(false, ts_syntax()))?
                .code)
        })
}

pub fn process_opts(minify: bool, syntax: Syntax) -> Options {
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

pub fn es_or_ts_syntax(p: &Path) -> Syntax {
    p.extension()
        .map(|ext| match ext.to_str().unwrap() {
            "ts" => ts_syntax(),
            _ => es_syntax(),
        })
        .unwrap()
}

pub fn es_syntax() -> Syntax {
    Syntax::Es(EsSyntax::default())
}

pub fn ts_syntax() -> Syntax {
    Syntax::Typescript(TsSyntax {
        decorators: false,
        disallow_ambiguous_jsx_like: true,
        tsx: false,
        dts: false,
        no_early_errors: false,
    })
}

pub fn es_target() -> EsVersion {
    EsVersion::EsNext
}

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

impl Default for SwcCompiler {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn fs_source_with_compiler<F, R>(self, p: &Path, f: F) -> CompileResult<R>
    where
        F: FnOnce(&Compiler, &Handler, Arc<SourceFile>) -> Result<R, anyhow::Error>,
    {
        let source_file = self.source_map.load_file(p)?;
        self.with_compiler(|compiler, handler| f(compiler, handler, source_file))
    }

    pub fn string_source_with_compiler<F, R>(self, p: PathBuf, js: String, f: F) -> CompileResult<R>
    where
        F: FnOnce(&Compiler, &Handler, Arc<SourceFile>) -> Result<R, anyhow::Error>,
    {
        let source_file = self.source_map.new_source_file(Arc::new(p.into()), js);
        self.with_compiler(|compiler, handler| f(compiler, handler, source_file))
    }

    pub fn with_compiler<F, R>(&self, f: F) -> CompileResult<R>
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
