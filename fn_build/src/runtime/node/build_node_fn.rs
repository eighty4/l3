use crate::runtime::build_fn::{build_fn_inner, BuildTask};
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
    match build_spec.mode {
        BuildMode::Debug => {
            build_tasks.push(BuildTask::CopySourceFiles(parse_manifest.source_paths()))
        }
        BuildMode::Release => {
            for source in &parse_manifest.sources {
                if let Some(extension) = source.path.extension() {
                    if extension == "js" || extension == "mjs" {
                        build_tasks.push(BuildTask::TransformSourceFile(source.path.clone()));
                        continue;
                    }
                }
                build_tasks.push(BuildTask::CopySourceFile(source.path.clone()));
            }
        }
    }
    build_fn_inner(&build_spec, parse_manifest, build_tasks, swc_transform).await
}

fn swc_transform(p: &Path) -> FnBuildResult<String> {
    Ok(SwcCompiler::new().minify_js(p)?)
}

impl From<CompileError> for FnBuildError {
    fn from(err: CompileError) -> Self {
        FnBuildError::ParseError(FnParseError::from(err))
    }
}
