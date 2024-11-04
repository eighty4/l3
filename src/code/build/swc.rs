use std::fs;
use std::path::Path;
use std::sync::Arc;

use swc::config::{Config, JsMinifyOptions, JscConfig, Options};
use swc::BoolConfig;
use swc_common::FileName;
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{Syntax, TsSyntax};

use crate::code::build::{BuildMode, Builder};
use crate::code::source::path::{FunctionBuildDir, SourcePath};
use crate::code::source::{Language, SourceFile};
use crate::code::swc::compiler::with_swc_compiler_without_diagnostics;
use crate::project::Lx3Project;

pub struct SwcBuilder {
    project: Arc<Lx3Project>,
}

impl SwcBuilder {
    pub fn new(project: Arc<Lx3Project>) -> Self {
        Self { project }
    }
}

impl Builder for SwcBuilder {
    fn build(
        &self,
        source_file: &SourceFile,
        build_dir: &FunctionBuildDir,
    ) -> Result<SourcePath, anyhow::Error> {
        debug_assert!(
            matches!(source_file.language, Language::JavaScript)
                || matches!(source_file.language, Language::TypeScript)
        );
        let compiled_result = match source_file.language {
            Language::TypeScript => Some(compile_ts_file(
                &source_file.path.abs,
                &self.project.build_mode,
            )?),
            _ => {
                if self.project.build_mode.should_minify() {
                    Some(minify_js_file(&source_file.path.abs)?)
                } else {
                    None
                }
            }
        };
        match compiled_result {
            Some(compiled) => {
                let build_dir_path = source_file.path.to_build_dir(build_dir.clone());
                let _ = fs::create_dir_all(build_dir_path.abs.parent().unwrap());
                fs::write(&build_dir_path.abs, compiled)?;
                Ok(build_dir_path)
            }
            None => Ok(source_file.path.clone()),
        }
    }
}

#[allow(unused)]
pub fn compile_ts(ts: String, mode: &BuildMode) -> Result<String, anyhow::Error> {
    with_swc_compiler_without_diagnostics::<_, String>(|compiler, handler, source_map| {
        let result = compiler.process_js_file(
            source_map.new_source_file(Arc::new(FileName::Anon), ts),
            handler,
            &compile_ts_options(mode),
        );
        Ok(result?.code)
    })
}

#[allow(unused)]
pub fn compile_ts_file(path: &Path, mode: &BuildMode) -> Result<String, anyhow::Error> {
    with_swc_compiler_without_diagnostics::<_, String>(|compiler, handler, source_map| {
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
    with_swc_compiler_without_diagnostics::<_, String>(|compiler, handler, source_map| {
        let result = compiler.minify(
            source_map.new_source_file(Arc::new(FileName::Anon), js),
            handler,
            &minify_options(),
            Default::default(),
        );
        Ok(result?.code)
    })
}

#[allow(unused)]
pub fn minify_js_file(path: &Path) -> Result<String, anyhow::Error> {
    with_swc_compiler_without_diagnostics::<_, String>(|compiler, handler, source_map| {
        let result = compiler.minify(
            source_map.load_file(path)?,
            handler,
            &minify_options(),
            Default::default(),
        );
        Ok(result?.code)
    })
}

fn minify_options() -> JsMinifyOptions {
    Default::default()
}
