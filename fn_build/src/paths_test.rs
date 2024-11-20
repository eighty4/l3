use crate::paths::join_file_paths;
use std::path::PathBuf;

#[test]
fn test_join_file_paths_joins_rel_path_with_pwd() {
    let base = PathBuf::from("lib/api.js");
    let relative = PathBuf::from("./data.js");
    let result = join_file_paths(&base, &relative);
    assert_eq!(result, PathBuf::from("lib/data.js"));
    assert_eq!(
        result.to_string_lossy().to_string(),
        String::from("lib/data.js")
    );
}

#[test]
fn test_join_file_paths_joins_rel_path_with_ancestor() {
    let base = PathBuf::from("fixtures/node/js/relative_import/routes/data/lambda.js");
    let relative = PathBuf::from("../../lib/data.js");
    let result = join_file_paths(&base, &relative);
    assert_eq!(
        result,
        PathBuf::from("fixtures/node/js/relative_import/lib/data.js")
    );
    assert_eq!(
        result.to_string_lossy().to_string(),
        String::from("fixtures/node/js/relative_import/lib/data.js")
    );
    assert!(result.is_file());
}

#[test]
fn test_join_file_paths_joins_rel_path_with_sibling() {
    let base = PathBuf::from("fixtures/node/js/relative_import/routes/data/lambda.js");
    let relative = PathBuf::from("../.././lib/data.js");
    let result = join_file_paths(&base, &relative);
    assert_eq!(
        result,
        PathBuf::from("fixtures/node/js/relative_import/lib/data.js")
    );
    assert_eq!(
        result.to_string_lossy().to_string(),
        String::from("fixtures/node/js/relative_import/lib/data.js")
    );
    assert!(result.is_file());
}
