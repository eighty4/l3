use std::path::PathBuf;

use crate::code::source::path::SourcePath;
use crate::testing::ProjectTest;

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
    // assert_eq!(data_src.abs, cwd.join("src/data.js"));
    // assert_eq!(data_src.rel, PathBuf::from("src/data.js"));
}

// #[test]
// fn test_source_path_to_relative_source_for_source_outside_project_dir() {
//     let project_test = ProjectTest::builder().build();
//     let rel = PathBuf::from("api.js");
//     let lambda_src = SourcePath::from_rel(&project_test.project_dir, rel);
//     let data_src = lambda_src.to_relative_source(&PathBuf::from("../data.js"));
//     assert_eq!(
//         data_src.abs,
//         project_test.project_dir.parent().unwrap().join("data.js")
//     );
//     assert_eq!(data_src.rel, PathBuf::from("../data.js"));
// }
