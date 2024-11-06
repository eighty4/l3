use crate::result::{FnBuild, FnSources};
use crate::spec::{FnBuildSpec, FnParseSpec};
use crate::swc::{build_js_fn, parse_js_fn};
use crate::testing::{run_fixtures, BuildProcess, BuildProcessResult};
use std::path::PathBuf;
use std::sync::Arc;

struct JavaScriptBuild {}

impl BuildProcess for JavaScriptBuild {
    fn build(&self, build_spec: FnBuildSpec) -> BuildProcessResult<FnBuild> {
        Box::pin(build_js_fn(build_spec))
    }

    fn parse(&self, parse_spec: FnParseSpec) -> BuildProcessResult<FnSources> {
        Box::pin(parse_js_fn(parse_spec))
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
