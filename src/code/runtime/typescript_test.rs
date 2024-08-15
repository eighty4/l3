use crate::code::runtime::typescript::read_typescript_config;
use crate::testing::{ProjectTest, TestSource};

#[test]
fn test_read_details_reads_path_aliases() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("tsconfig.json")
                .content("{\"compilerOptions\":{\"paths\":{\"@app/*\":[\"./src/*\"]}}}"),
        )
        .build();
    let ts_deets = read_typescript_config(&project_test.project_dir);
    assert_eq!(1, ts_deets.path_aliases.len());
    assert!(ts_deets.path_aliases.get("@app/*").is_some());
    assert_eq!(
        "./src/*",
        ts_deets
            .path_aliases
            .get("@app/*")
            .unwrap()
            .get(0)
            .unwrap()
            .to_string_lossy()
            .as_ref()
    );
}
