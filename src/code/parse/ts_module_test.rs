use std::path::PathBuf;

use crate::code::parse::parse_source_file;
use crate::testing::{ProjectTest, TestSource};

#[test]
fn test_ts_module_parse_source_file_not_yet_supported() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("lambda.ts").content("export function GET(arg: string) {}"),
        )
        .build();
    assert!(
        parse_source_file(&PathBuf::from("lambda.ts"), &project_test.project_dir)
            .err()
            .unwrap()
            .to_string()
            .starts_with("ts is not a supported file type for source file ")
    );
}
