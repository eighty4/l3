use crate::runtime::build_fn::{build_fn_inner, BuildTask, TransformResult};
use crate::runtime::node::parse_node_fn;
use crate::swc::compiler::{CompileError, SwcCompiler};
use crate::{
    BuildMode, FnBuildError, FnBuildManifest, FnBuildResult, FnBuildSpec, FnDependencies,
    FnParseError,
};
use std::path::{Path, PathBuf};

pub async fn build_node_fn(build_spec: FnBuildSpec) -> FnBuildResult<FnBuildManifest> {
    let parse_manifest = parse_node_fn(build_spec.to_parse_spec()).await?;
    let mut build_tasks = Vec::new();
    if let FnDependencies::Required = parse_manifest.dependencies {
        build_tasks.push(BuildTask::CopyDirectoryRecursively(PathBuf::from(
            "node_modules",
        )));
    }
    let mut ts = false;
    let mut copy_sources: Vec<PathBuf> = Vec::new();
    match build_spec.mode {
        BuildMode::Debug => {
            for p in parse_manifest.source_paths() {
                if is_ts(&p) {
                    ts = true;
                    build_tasks.push(BuildTask::TransformSourceFile(p));
                } else {
                    copy_sources.push(p)
                }
            }
        }
        BuildMode::Release => {
            for p in parse_manifest.source_paths() {
                if is_ts_or_es(&p) {
                    ts = true;
                    build_tasks.push(BuildTask::TransformSourceFile(p));
                } else {
                    copy_sources.push(p)
                }
            }
        }
    }
    build_tasks.push(BuildTask::CopySourceFiles(copy_sources));
    build_fn_inner(
        &build_spec,
        parse_manifest,
        build_tasks,
        match (ts, &build_spec.mode) {
            (true, BuildMode::Debug) => swc_transpile,
            (true, BuildMode::Release) => swc_transpile_and_minify,
            (false, BuildMode::Release) => swc_minify,
            _ => noop,
        },
    )
    .await
}

fn noop(_p: &Path, _s: String) -> FnBuildResult<TransformResult> {
    panic!();
}

fn swc_minify(p: &Path, s: String) -> FnBuildResult<TransformResult> {
    Ok(TransformResult::RetainPath(
        SwcCompiler::new().minify_js(p.to_path_buf(), s)?,
    ))
}

// if not minifying, js/mjs should be copied as is so this transform does not check for ts or es like swc_transpile_and_minify
fn swc_transpile(p: &Path, s: String) -> FnBuildResult<TransformResult> {
    Ok(TransformResult::RewriteExt(
        SwcCompiler::new().transpile_ts(p.to_path_buf(), s)?,
        "js".into(),
    ))
}

fn swc_transpile_and_minify(p: &Path, s: String) -> FnBuildResult<TransformResult> {
    if is_ts(p) {
        Ok(TransformResult::RewriteExt(
            SwcCompiler::new().transpile_and_minify_ts(p.to_path_buf(), s)?,
            "js".into(),
        ))
    } else {
        Ok(TransformResult::RetainPath(
            SwcCompiler::new().minify_js(p.to_path_buf(), s)?,
        ))
    }
}

fn is_ts_or_es(p: &Path) -> bool {
    if let Some(extension) = p.extension() {
        if extension == "ts" || extension == "js" || extension == "mjs" {
            return true;
        }
    }
    false
}

fn is_ts(p: &Path) -> bool {
    if let Some(extension) = p.extension() {
        if extension == "ts" {
            return true;
        }
    }
    false
}

impl From<CompileError> for FnBuildError {
    fn from(err: CompileError) -> Self {
        FnBuildError::ParseError(FnParseError::from(err))
    }
}
