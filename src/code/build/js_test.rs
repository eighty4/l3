use crate::code::build::LambdaFnBuild;
use crate::lambda::HttpMethod::Get;
use crate::lambda::{LambdaFn, RouteKey};
use crate::testing::project::ProjectTest;
use crate::testing::source::TestSource;

#[tokio::test]
async fn test_js_build_relative_import_with_explicit_extension() {
    for ext in ["js", "mjs"] {
        let project_test = ProjectTest::builder()
            .with_sources(vec![
                TestSource::with_path(format!("routes/data/lambda.{ext}").as_str())
                    .content(format!("import {{getData}} from '../../lib/data.{ext}';export const GET = () => console.log(getData())").as_str()),
                TestSource::with_path(format!("lib/data.{ext}").as_str()).content("export const getData = () => 'data'"),
                TestSource::with_path("package.json").content("{\"type\":\"module\"}")
            ])
            .verify_with_runtime()
            .build();

        let route_key = RouteKey::new(Get, "data".to_string());
        let lambda_fn = LambdaFn::new(
            "GET".to_string(),
            project_test.source_path(format!("routes/data/lambda.{ext}").as_str()),
            project_test.project.clone(),
            route_key,
        );

        let lambda_build = LambdaFnBuild::in_api_dir(lambda_fn, project_test.project.clone());
        let build_manifest = lambda_build.build().await.unwrap();
        assert_eq!(build_manifest.fn_sources.len(), 3);
    }
}
