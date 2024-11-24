mod archive;
mod fs;
mod paths;
mod result;
mod routing;
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

#[cfg(test)]
mod routing_test;

pub use crate::result::*;
pub use crate::routing::*;
pub use crate::spec::*;

/// Builds a lambda function.
pub async fn build_fn(build_spec: FnBuildSpec) -> FnBuildResult<FnBuildManifest> {
    debug_assert!(build_spec.entrypoint.is_relative());
    debug_assert!(build_spec.entrypoint.parent().is_some());
    debug_assert!(build_spec.output.build_root.is_absolute());
    match build_spec.entrypoint.extension() {
        None => Err(FnBuildError::from(FnParseError::InvalidFileType)),
        Some(extension) => match extension.to_string_lossy().as_ref() {
            "js" | "mjs" => runtime::node::build_node_fn(build_spec).await,
            "py" => runtime::python::build_python_fn(build_spec).await,
            "ts" => todo!(),
            &_ => Err(FnBuildError::from(FnParseError::InvalidFileType)),
        },
    }
}

/// Parses a source file for exported functions that could be lambdas.
pub async fn parse_entrypoint(parse_spec: FnParseSpec) -> FnParseResult<Vec<FnHandler>> {
    debug_assert!(parse_spec.entrypoint.is_relative());
    debug_assert!(parse_spec.entrypoint.parent().is_some());
    debug_assert!(parse_spec.project_dir.is_absolute());
    debug_assert!(parse_spec.project_dir.is_dir());
    match parse_spec.entrypoint.extension() {
        None => Err(FnParseError::InvalidFileType),
        Some(extension) => match extension.to_string_lossy().as_ref() {
            "js" | "mjs" => runtime::node::parse_node_entrypoint(parse_spec).await,
            "py" => runtime::python::parse_python_entrypoint(parse_spec).await,
            "ts" => todo!(),
            &_ => Err(FnParseError::InvalidFileType),
        },
    }
}

/// Pareses the source tree of a lambda.
pub async fn parse_fn(parse_spec: FnParseSpec) -> FnParseResult<FnParseManifest> {
    debug_assert!(parse_spec.entrypoint.is_relative());
    debug_assert!(parse_spec.entrypoint.parent().is_some());
    debug_assert!(parse_spec.project_dir.is_absolute());
    debug_assert!(parse_spec.project_dir.is_dir());
    match parse_spec.entrypoint.extension() {
        None => Err(FnParseError::InvalidFileType),
        Some(extension) => match extension.to_string_lossy().as_ref() {
            "js" | "mjs" => runtime::node::parse_node_fn(parse_spec).await,
            "py" => runtime::python::parse_python_fn(parse_spec).await,
            "ts" => todo!(),
            &_ => Err(FnParseError::InvalidFileType),
        },
    }
}
