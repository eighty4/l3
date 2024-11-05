mod result;
mod spec;
mod swc;

#[cfg(test)]
mod build_test;

#[cfg(test)]
mod parse_test;

#[cfg(test)]
mod testing;

use crate::result::{FnBuild, FnBuildError, FnBuildResult, FnSource};
use crate::spec::{FnBuildSpec, FnParseSpec};
use crate::swc::{build_js_fn, parse_js_fn};

pub async fn build_fn(build_spec: FnBuildSpec) -> FnBuildResult<FnBuild> {
    debug_assert!(build_spec.entrypoint.is_relative());
    debug_assert!(build_spec.entrypoint.parent().is_some());
    debug_assert!(build_spec.output.is_absolute());
    debug_assert!(build_spec.output.is_dir());
    debug_assert!(build_spec.project_dir.is_absolute());
    debug_assert!(build_spec.project_dir.is_dir());
    match build_spec.entrypoint.extension() {
        None => Err(FnBuildError::InvalidFileType),
        Some(extension) => match extension.to_string_lossy().as_ref() {
            "js" | "mjs" => build_js_fn(build_spec).await,
            "py" => todo!(),
            "ts" => todo!(),
            &_ => Err(FnBuildError::InvalidFileType),
        },
    }
}

pub async fn parse_fn(parse_spec: FnParseSpec) -> FnBuildResult<FnSource> {
    debug_assert!(parse_spec.entrypoint.is_relative());
    debug_assert!(parse_spec.entrypoint.parent().is_some());
    debug_assert!(parse_spec.project_dir.is_absolute());
    debug_assert!(parse_spec.project_dir.is_dir());
    match parse_spec.entrypoint.extension() {
        None => Err(FnBuildError::InvalidFileType),
        Some(extension) => match extension.to_string_lossy().as_ref() {
            "js" | "mjs" => parse_js_fn(parse_spec).await,
            "py" => todo!(),
            "ts" => todo!(),
            &_ => Err(FnBuildError::InvalidFileType),
        },
    }
}
