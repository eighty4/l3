use crate::runtime::build_fn::{build_fn_inner, BuildTask, TransformResult};
use crate::runtime::python::parse_python_fn;
use crate::{FnBuildManifest, FnBuildResult, FnBuildSpec, FnDependencies};
use std::path::Path;

pub async fn build_python_fn(build_spec: FnBuildSpec) -> FnBuildResult<FnBuildManifest> {
    let parse_manifest = parse_python_fn(build_spec.to_parse_spec()).await?;
    match parse_manifest.dependencies {
        FnDependencies::Unused => {}
        _ => todo!(),
    };
    let build_tasks = vec![BuildTask::CopySourceFiles(parse_manifest.source_paths())];
    build_fn_inner(&build_spec, parse_manifest, build_tasks, noop_transform).await
}

fn noop_transform(_: &Path, _: String) -> FnBuildResult<TransformResult> {
    Ok(TransformResult::RetainPath("".to_string()))
}
