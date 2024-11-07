use crate::paths::join_file_paths;
use std::path::PathBuf;

#[test]
fn test_join_file_paths_joins_rel_path_with_ancestor() {
    let base = PathBuf::from("fixtures/node/js/relative_import/routes/data/lambda.js");
    let relative = PathBuf::from("../../lib/data.js");
    let result = join_file_paths(&base, &relative);
    assert_eq!(
        join_file_paths(&base, &relative),
        PathBuf::from("fixtures/node/js/relative_import/lib/data.js")
    );
    assert!(result.is_file());
}

#[test]
fn test_join_file_paths_joins_rel_path_with_sibling() {
    let base = PathBuf::from("fixtures/node/js/relative_import/routes/data/lambda.js");
    let relative = PathBuf::from("../.././lib/data.js");
    let result = join_file_paths(&base, &relative);
    assert_eq!(
        join_file_paths(&base, &relative),
        PathBuf::from("fixtures/node/js/relative_import/lib/data.js")
    );
    assert!(result.is_file());
}
