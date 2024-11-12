use crate::build_fn;
use crate::runtime::Runtime;
use crate::FnBuildError;
use crate::{BuildMode, FnBuildSpec, FnParseSpec};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use temp_dir::TempDir;

#[tokio::test]
async fn build_fn_errors_for_invalid_extension() {
    let build_dir = TempDir::new().unwrap();
    for entrypoint in &["README", "README.md"] {
        let build_spec = FnBuildSpec {
            function: FnParseSpec {
                entrypoint: PathBuf::from(entrypoint),
                project_dir: Arc::new(
                    env::current_dir()
                        .unwrap()
                        .join("fixtures/node/js/http_route"),
                ),
                runtime: Runtime::Node(Default::default()),
            },
            mode: BuildMode::Debug,
            output: build_dir.path().to_path_buf(),
        };
        match build_fn(build_spec).await {
            Err(FnBuildError::InvalidFileType) => {}
            _ => panic!(),
        };
    }
}
