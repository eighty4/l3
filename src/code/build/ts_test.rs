use crate::code::build::LambdaFnBuild;
use crate::lambda::HttpMethod::Get;
use crate::lambda::{LambdaFn, RouteKey};
use crate::testing::project::ProjectTest;
use crate::testing::source::TestSource;

#[tokio::test]
async fn test_ts_build_relative_import() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.ts").content("import {getData} from '../../lib/data.js';export const GET = () => console.log(getData())"),
        )
        .with_source(TestSource::with_path("lib/data.ts").content("export const getData = () => 'data'"))
        .write_project_sources("code-build-ts_test-test_ts_build_relative_import")
        .build();

    let route_key = RouteKey::new(Get, "data".to_string());
    let lambda_fn = LambdaFn::new(
        "GET".to_string(),
        project_test.source_path("routes/data/lambda.ts"),
        project_test.project.clone(),
        route_key,
    );

    let lambda_build = LambdaFnBuild::in_api_dir(lambda_fn, project_test.project.clone());
    let build_manifest = lambda_build.build().await.unwrap();
    assert_eq!(build_manifest.fn_sources.len(), 2);
}
