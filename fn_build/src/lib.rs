mod result;
mod swc;

#[cfg(test)]
mod parse_test;

use crate::result::{FnBuildError, FnBuildResult, FnSource};
use crate::swc::parse_js_fn;
use std::path::PathBuf;

pub async fn parse_fn(entrypoint: PathBuf) -> FnBuildResult<FnSource> {
    match entrypoint.extension() {
        None => Err(FnBuildError::InvalidFileType),
        Some(extension) => match extension.to_string_lossy().as_ref() {
            "js" | "mjs" => parse_js_fn(entrypoint).await,
            "py" => todo!(),
            "ts" => todo!(),
            &_ => Err(FnBuildError::InvalidFileType),
        },
    }
}
