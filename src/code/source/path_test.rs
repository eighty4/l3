use crate::code::build::BuildMode;
use crate::code::source::path::{FunctionBuildDir, SourcePath};
use crate::testing::ProjectTest;
use std::path::PathBuf;

#[test]
fn test_function_build_dir_for_debug_build() {
    let project_test = ProjectTest::builder().build_mode(BuildMode::Debug).build();
    let build_dir =
        FunctionBuildDir::new(&project_test.project_deets, &"l3-get-data-fn".to_string());
    let expected_rel = ".l3/aws/API_ID/l3-get-data-fn/debug";
    assert_eq!(build_dir._rel.to_string_lossy().as_ref(), expected_rel);
    assert_eq!(build_dir.abs, project_test.project_dir.join(expected_rel));
}

#[test]
fn test_function_build_dir_for_release_build() {
    let project_test = ProjectTest::builder()
        .build_mode(BuildMode::Release)
        .build();
    let build_dir =
        FunctionBuildDir::new(&project_test.project_deets, &"l3-get-data-fn".to_string());
    let expected_rel = ".l3/aws/API_ID/l3-get-data-fn/release";
    assert_eq!(build_dir._rel.to_string_lossy().as_ref(), expected_rel);
    assert_eq!(build_dir.abs, project_test.project_dir.join(expected_rel));
}

#[test]
fn test_source_path_to_relative_source() {
    let project_test = ProjectTest::builder().build();
    let rel = PathBuf::from("routes/data/lambda.js");
    let lambda_src = SourcePath::from_rel(&project_test.project_dir, rel);
    let data_src = lambda_src.to_relative_source(&PathBuf::from("../../src/data.js"));
    assert_eq!(
        data_src.abs,
        project_test
            .project_dir
            .join("routes/data/../../src/data.js")
    );
    assert_eq!(data_src.rel, PathBuf::from("routes/data/../../src/data.js"));
}
