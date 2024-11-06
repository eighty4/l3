use crate::paths::rewrite_current_and_parent_path_segments;
use std::path::PathBuf;

#[test]
fn test_rewrite_current_and_parent_path_segments_with_absolute_path() {
    let result = rewrite_current_and_parent_path_segments(PathBuf::from(
        "/user/project/routes/data/../../src/data.js",
    ));
    assert_eq!(result, PathBuf::from("/user/project/src/data.js"));
}

#[test]
fn test_rewrite_current_and_parent_path_segments_with_relative_path() {
    let result =
        rewrite_current_and_parent_path_segments(PathBuf::from("routes/data/../../src/data.js"));
    assert_eq!(result, PathBuf::from("src/data.js"));
}

#[test]
fn test_rewrite_current_and_parent_path_segments_with_current_path_segment() {
    let result = rewrite_current_and_parent_path_segments(PathBuf::from(
        "/user/project/routes/data/.././lambda.js",
    ));
    assert_eq!(result, PathBuf::from("/user/project/routes/lambda.js"));
}

#[test]
fn test_rewrite_current_and_parent_path_segments_for_noop() {
    let result = rewrite_current_and_parent_path_segments(PathBuf::from("routes/data/lambda.js"));
    assert_eq!(result, PathBuf::from("routes/data/lambda.js"));
}
