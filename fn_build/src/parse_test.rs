use crate::{parse_fn, FnBuildError};
use std::path::PathBuf;

#[tokio::test]
async fn parse_fn_errors_for_invalid_extension() {
    match parse_fn(PathBuf::from("README.md")).await {
        Err(FnBuildError::InvalidFileType) => {}
        _ => panic!(),
    };
}

#[tokio::test]
async fn parse_fn_errors_without_extension() {
    match parse_fn(PathBuf::from("README")).await {
        Err(FnBuildError::InvalidFileType) => {}
        _ => panic!(),
    };
}
