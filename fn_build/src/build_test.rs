use crate::build_fn;
use crate::result::FnBuildError;
use crate::spec::{BuildMode, FnBuildSpec};
use std::env;
use std::path::PathBuf;
use temp_dir::TempDir;

#[tokio::test]
async fn build_fn_errors_for_invalid_extension() {
    let build_dir = TempDir::new().unwrap();
    for entrypoint in &["README", "README.md"] {
        let build_spec = FnBuildSpec {
            entrypoint: PathBuf::from(entrypoint),
            mode: BuildMode::Debug,
            output: build_dir.path().to_path_buf(),
            project_dir: env::current_dir()
                .unwrap()
                .join("fixtures/swc/nodejs/js/http_route"),
        };
        match build_fn(build_spec).await {
            Err(FnBuildError::InvalidFileType) => {}
            _ => panic!(),
        };
    }
}
