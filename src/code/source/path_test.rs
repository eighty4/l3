use crate::code::build::BuildMode;
use crate::code::source::path::{rewrite_current_and_parent_in_path, FunctionBuildDir, SourcePath};
use crate::lambda::{HttpMethod, LambdaFn, RouteKey};
use crate::testing::project::ProjectTest;
use crate::testing::source::TestSource;
use std::path::PathBuf;

#[test]
fn test_source_path_is_lambda_file_name() {
    for p in &[
        "routes/data/lambda.js",
        "routes/data/lambda.mjs",
        "routes/data/lambda.ts",
        "routes/data/lambda.py",
    ] {
        assert!(SourcePath::is_lambda_file_name(&PathBuf::from(p)));
    }
    for p in &["routes/data/lambda.go", "routes/data/.env"] {
        assert!(!SourcePath::is_lambda_file_name(&PathBuf::from(p)));
    }
}

#[tokio::test]
async fn test_function_build_dir_for_debug_build() {
    let project_test = ProjectTest::builder()
        .api_id("api_id")
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export function GET(){}"),
        )
        .build_mode(BuildMode::Debug)
        .build();
    let lambda_fn = LambdaFn::new(
        "GET".to_string(),
        project_test.source_path("routes/data/lambda.js"),
        project_test.project.clone(),
        RouteKey::new(HttpMethod::Get, "data".to_string()),
    );
    let build_dir = FunctionBuildDir::PlatformSync(project_test.project.clone(), lambda_fn);
    assert_eq!(
        build_dir.to_path(),
        project_test
            .project_dir
            .join(".l3/aws/api_id/l3-PROJECT_NAME-data-get/debug")
    );
}

// #[tokio::test]
// async fn test_function_build_dir_for_release_build() {
//     let project_test = ProjectTest::builder()
//         .build_mode(BuildMode::Release)
//         .build();
//     let build_dir = FunctionBuildDir::new(&project_test.project, &"l3-get-data-fn".to_string());
//     let expected_rel = ".l3/aws/API_ID/l3-get-data-fn/release";
//     assert_eq!(build_dir._rel.to_string_lossy().as_ref(), expected_rel);
//     assert_eq!(build_dir.abs, project_test.project_dir.join(expected_rel));
// }

#[tokio::test]
async fn test_source_path_to_relative_source() {
    let project_test = ProjectTest::builder().build();
    let rel = PathBuf::from("routes/data/lambda.js");
    let lambda_src = SourcePath::from_rel(&project_test.project_dir, rel);
    let data_src = lambda_src.to_relative_source(&PathBuf::from("../../src/data.js"));
    assert_eq!(
        data_src.abs(),
        &project_test.project_dir.join("src/data.js")
    );
    assert_eq!(data_src.rel(), &PathBuf::from("src/data.js"));
}

#[test]
fn test_rewrite_current_and_parent_path_components_with_absolute_path() {
    let result = rewrite_current_and_parent_in_path(&PathBuf::from(
        "/user/project/routes/data/../../src/data.js",
    ));
    assert!(result.is_ok());
    let option = result.unwrap();
    assert!(option.is_some());
    let value = option.unwrap();
    assert_eq!(value, PathBuf::from("/user/project/src/data.js"));
}

#[test]
fn test_rewrite_current_and_parent_path_components_with_relative_path() {
    let result =
        rewrite_current_and_parent_in_path(&PathBuf::from("routes/data/../../src/data.js"));
    assert!(result.is_ok());
    let option = result.unwrap();
    assert!(option.is_some());
    let value = option.unwrap();
    assert_eq!(value, PathBuf::from("src/data.js"));
}

#[test]
fn test_rewrite_current_and_parent_path_components_for_noop() {
    let result = rewrite_current_and_parent_in_path(&PathBuf::from("routes/data/lambda.js"));
    assert!(result.is_ok());
    let option = result.unwrap();
    assert!(option.is_none());
}
