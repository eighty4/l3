use crate::code::source::tracker::{SourceTracker, SourceUpdate};
use crate::testing::{ProjectTest, TestSource};
use tokio::sync::mpsc;

#[test]
fn test_lambda_sources_builds_source_tree() {
    let path = "routes/data/lambda.js";
    let project_test = ProjectTest::builder()
        .with_source(TestSource::with_path(path))
        .build();
    let (sender, receiver) = mpsc::channel::<SourceUpdate>(100);
    let mut sources = SourceTracker::new(project_test.project_dir.clone(), sender);
    sources.file_created(project_test.path(path));
}
