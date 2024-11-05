use crate::spec::FnBuildSpec;
use crate::swc::build_js_fn;
use crate::testing::{run_fixtures, BuildProcess, BuildProcessResult};
use std::path::PathBuf;
use std::sync::Arc;

struct JavaScriptBuild {}

impl BuildProcess for JavaScriptBuild {
    fn build(&self, build_spec: FnBuildSpec) -> BuildProcessResult {
        Box::pin(build_js_fn(build_spec))
    }
}

#[tokio::test]
pub async fn test_nodejs_js_fixtures() {
    run_fixtures(
        Arc::new(Box::new(JavaScriptBuild {})),
        PathBuf::from("fixtures/swc/nodejs/js"),
    )
    .await;
}
