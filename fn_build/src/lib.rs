mod archive;
mod fs;
mod paths;
mod result;
pub mod runtime;
mod spec;
mod swc;

#[cfg(test)]
mod build_test;

#[cfg(test)]
mod parse_test;

#[cfg(test)]
mod paths_test;

#[cfg(test)]
mod testing;

pub use crate::result::*;
pub use crate::spec::*;

pub async fn build_fn(build_spec: FnBuildSpec) -> FnBuildResult<FnBuild> {
    debug_assert!(build_spec.function.entrypoint.is_relative());
    debug_assert!(build_spec.function.entrypoint.parent().is_some());
    debug_assert!({
        let output_dir = match &build_spec.output {
            FnBuildOutput::Archive { build_root, .. } => build_root,
            FnBuildOutput::Directory(dir_path) => dir_path,
        };
        output_dir.is_absolute() && output_dir.is_dir()
    });
    match build_spec.function.entrypoint.extension() {
        None => Err(FnBuildError::InvalidFileType),
        Some(extension) => match extension.to_string_lossy().as_ref() {
            "js" | "mjs" => runtime::node::build_node_fn(build_spec).await,
            "py" => todo!(),
            "ts" => todo!(),
            &_ => Err(FnBuildError::InvalidFileType),
        },
    }
}

pub async fn parse_fn(parse_spec: FnParseSpec) -> FnBuildResult<FnManifest> {
    debug_assert!(parse_spec.entrypoint.is_relative());
    debug_assert!(parse_spec.entrypoint.parent().is_some());
    debug_assert!(parse_spec.project_dir.is_absolute());
    debug_assert!(parse_spec.project_dir.is_dir());
    match parse_spec.entrypoint.extension() {
        None => Err(FnBuildError::InvalidFileType),
        Some(extension) => match extension.to_string_lossy().as_ref() {
            "js" | "mjs" => runtime::node::parse_node_fn(parse_spec).await,
            "py" => todo!(),
            "ts" => todo!(),
            &_ => Err(FnBuildError::InvalidFileType),
        },
    }
}
