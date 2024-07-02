use std::path::PathBuf;

use crate::code::parse::parse_source_file;
use crate::testing::{ProjectTest, TestSource};

#[test]
fn test_parse_module_errors_for_unsupported_source_type() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("lambda.toml").content("[unsupported]"))
        .build();
    assert!(
        parse_source_file(&PathBuf::from("lambda.toml"), &project_test.project_dir)
            .err()
            .unwrap()
            .to_string()
            .starts_with("toml is not a supported file type for source file ")
    );
}
