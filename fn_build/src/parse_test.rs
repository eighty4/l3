use crate::parse_fn;
use crate::runtime::Runtime;
use crate::{FnParseError, FnParseSpec};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::test]
async fn parse_fn_errors_for_invalid_extension() {
    for entrypoint in &["README", "README.md"] {
        let parse_spec = FnParseSpec {
            entrypoint: PathBuf::from(entrypoint),
            project_dir: Arc::new(
                env::current_dir()
                    .unwrap()
                    .join("fixtures/node/js/http_route"),
            ),
            runtime: Runtime::Node(Default::default()),
        };
        match parse_fn(parse_spec).await {
            Err(FnParseError::InvalidFileType) => {}
            _ => panic!(),
        };
    }
}
