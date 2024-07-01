use crate::code::parse::parse_module_for_lambda_handlers;
use crate::testing::{ProjectTest, TestSource};

#[test]
fn test_parse_ts_module_not_yet_supported() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("lambda.ts").content("export function GET(arg: string) {}"),
        )
        .build();
    let module_path = project_test.path("lambda.ts");
    assert!(parse_module_for_lambda_handlers(&module_path)
        .unwrap_err()
        .to_string()
        .starts_with("ts is not a supported file type for source file "));
}
