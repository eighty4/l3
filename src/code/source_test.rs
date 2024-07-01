use std::path::PathBuf;

use crate::code::source::SourceFile;
use crate::testing::{ProjectTest, TestSource};

const SOURCE: &str = "export function GET(){}";

#[test]
fn test_source_file_new_lambda_file() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("routes/data/lambda.js").content(SOURCE))
        .build();
    let source_file = SourceFile::create(
        PathBuf::from("routes/data/lambda.js"),
        project_test.project_dir,
    )
    .unwrap();
    assert_eq!(source_file.extension, "js");
    assert_eq!(source_file.file_name, "lambda.js");
    assert_eq!(source_file.path, PathBuf::from("routes/data/lambda.js"));
}

#[test]
fn test_source_file_new_env_file() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("routes/data/.env").content("PROP=more data"))
        .build();
    let source_file =
        SourceFile::create(PathBuf::from("routes/data/.env"), project_test.project_dir).unwrap();
    assert_eq!(source_file.extension, "env");
    assert_eq!(source_file.file_name, ".env");
    assert_eq!(source_file.path, PathBuf::from("routes/data/.env"));
}
