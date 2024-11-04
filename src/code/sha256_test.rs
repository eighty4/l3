use crate::code::sha256::make_checksum;
use crate::testing::project::ProjectTest;
use crate::testing::source::TestSource;

#[tokio::test]
async fn test() {
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path("file").content("content"))
        .build();
    let result = make_checksum(&project_test.path("file"));
    assert!(result.is_ok());
    assert_eq!(
        "7XACtDnprIRfIjV9giusFERzD722AW0+yUMil7nsn3M=",
        result.unwrap()
    );
}
