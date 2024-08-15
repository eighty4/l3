use crate::code::parse::parse_source_file;
use crate::testing::{ProjectTest, TestSource};

#[test]
fn test_parse_module_errors_for_unsupported_source_type() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("lambda.toml").content("[unsupported]"))
        .build();
    assert_eq!(
        parse_source_file(project_test.source_path("lambda.toml"), Default::default())
            .err()
            .unwrap()
            .to_string(),
        "lambda.toml is not a supported file type"
    );
}
