use crate::runtime::Runtime;
use crate::spec::FnParseSpec;
use crate::{parse_fn, FnBuildError};
use std::env;
use std::path::PathBuf;

#[tokio::test]
async fn parse_fn_errors_for_invalid_extension() {
    for entrypoint in &["README", "README.md"] {
        let parse_spec = FnParseSpec {
            entrypoint: PathBuf::from(entrypoint),
            project_dir: env::current_dir()
                .unwrap()
                .join("fixtures/node/js/http_route"),
            runtime: Runtime::Node(Default::default()),
        };
        match parse_fn(parse_spec).await {
            Err(FnBuildError::InvalidFileType) => {}
            _ => panic!(),
        };
    }
}
