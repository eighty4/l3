use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::anyhow;
use swc::config::{Config, JsMinifyOptions, JscConfig, Options};
use swc::{try_with_handler, BoolConfig, Compiler};
use swc_common::errors::Handler;
use swc_common::{FileName, SourceMap, GLOBALS};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{Syntax, TsSyntax};

use crate::code::build::{BuildMode, BuildOptions, Builder};
use crate::code::source::path::SourcePath;
use crate::code::source::{Language, SourceFile};

pub struct SwcBuilder {}

impl SwcBuilder {
    pub fn new() -> Self {
        SwcBuilder {}
    }
}

impl Builder for SwcBuilder {
    fn build(
        &self,
        source_file: &SourceFile,
        options: &BuildOptions,
    ) -> Result<SourcePath, anyhow::Error> {
        debug_assert!(
            source_file.language == Language::JavaScript
                || source_file.language == Language::TypeScript
        );
        let compiled_result = if source_file.language == Language::TypeScript {
            Some(compile_ts_file(&source_file.path.abs, &options.mode)?)
        } else if options.mode.should_minify() {
            Some(minify_js_file(&source_file.path.abs)?)
        } else {
            None
        };
        let build_dir_path = source_file
            .path
            .to_build_dir(options.build_dir.clone(), &options.project_dir);
        _ = fs::create_dir_all(build_dir_path.abs.parent().unwrap());
        match compiled_result {
            Some(compiled) => {
                fs::write(&build_dir_path.abs, compiled)?;
                Ok(build_dir_path)
            }
            None => Ok(source_file.path.clone()),
        }
    }
}

#[allow(unused)]
pub fn compile_ts(ts: String, mode: &BuildMode) -> Result<String, anyhow::Error> {
    with_swc_compiler::<_, String>(|compiler, handler, source_map| {
        let result = compiler.process_js_file(
            source_map.new_source_file(FileName::Anon, ts),
            handler,
            &compile_ts_options(mode),
        );
        Ok(result?.code)
    })
}

#[allow(unused)]
pub fn compile_ts_file(path: &Path, mode: &BuildMode) -> Result<String, anyhow::Error> {
    with_swc_compiler::<_, String>(|compiler, handler, source_map| {
        let result = compiler.process_js_file(
            source_map.load_file(path)?,
            handler,
            &compile_ts_options(mode),
        );
        Ok(result?.code)
    })
}

#[allow(unused)]
fn compile_ts_options(mode: &BuildMode) -> Options {
    Options {
        config: Config {
            jsc: JscConfig {
                minify: match mode {
                    BuildMode::Debug => None,
                    BuildMode::Release => Some(minify_options()),
                },
                syntax: Some(Syntax::Typescript(TsSyntax::default())),
                target: Some(EsVersion::EsNext),
                ..Default::default()
            },
            minify: BoolConfig::from(mode.should_minify()),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[allow(unused)]
pub fn minify_js(js: String) -> Result<String, anyhow::Error> {
    with_swc_compiler::<_, String>(|compiler, handler, source_map| {
        let result = compiler.minify(
            source_map.new_source_file(FileName::Anon, js),
            handler,
            &minify_options(),
        );
        Ok(result?.code)
    })
}

#[allow(unused)]
pub fn minify_js_file(path: &Path) -> Result<String, anyhow::Error> {
    with_swc_compiler::<_, String>(|compiler, handler, source_map| {
        let result = compiler.minify(source_map.load_file(path)?, handler, &minify_options());
        Ok(result?.code)
    })
}

fn minify_options() -> JsMinifyOptions {
    Default::default()
}

#[allow(unused)]
fn with_swc_compiler<F, R>(f: F) -> Result<R, anyhow::Error>
where
    F: FnOnce(&Compiler, &Handler, Arc<SourceMap>) -> Result<R, anyhow::Error>,
{
    let source_map = Arc::<SourceMap>::default();
    let compiler = Compiler::new(source_map.clone());
    GLOBALS
        .set(&Default::default(), || {
            try_with_handler(source_map.clone(), Default::default(), |handler| {
                f(&compiler, handler, source_map)
            })
        })
        .map_err(|err| anyhow!("swc compiler error: {}", err.to_string()))
}
