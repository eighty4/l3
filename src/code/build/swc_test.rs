use crate::code::build::swc::{compile_ts, compile_ts_file, minify_js, minify_js_file, SwcBuilder};
use crate::code::build::{BuildMode, Builder};
use crate::code::source::path::{FunctionBuildDir, SourceKind};
use crate::lambda::{HttpMethod, LambdaFn, RouteKey};
use crate::testing::project::ProjectTest;
use crate::testing::source::TestSource;
use std::fs;
use std::path::PathBuf;

#[tokio::test]
async fn test_swc_builder_development_mode_returns_original_source_path() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export const GET = (e) => {}"),
        )
        .build();
    let source_file = &project_test.source_file("routes/data/lambda.js");
    let lambda_fn = LambdaFn::new(
        "GET".to_string(),
        project_test.source_path("routes/data/lambda.js"),
        project_test.project.clone(),
        RouteKey::new(HttpMethod::Get, "data".to_string()),
    );
    let build_dir = FunctionBuildDir::PlatformSync(project_test.project.clone(), lambda_fn);
    let built_path = SwcBuilder::new(project_test.project.clone())
        .build(source_file, &build_dir)
        .unwrap();
    assert!(matches!(built_path.kind, SourceKind::OriginalSource));
    assert_eq!(built_path.rel, PathBuf::from("routes/data/lambda.js"));
    assert_eq!(
        built_path.abs,
        project_test.project_dir.join("routes/data/lambda.js")
    );
}

#[tokio::test]
async fn test_swc_builder_development_mode_writes_ts_to_build_dir() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.ts")
                .content("export const GET = (e: string) => {}"),
        )
        .build();
    let source_file = &project_test.source_file("routes/data/lambda.ts");
    let lambda_fn = LambdaFn::new(
        "GET".to_string(),
        project_test.source_path("routes/data/lambda.js"),
        project_test.project.clone(),
        RouteKey::new(HttpMethod::Get, "data".to_string()),
    );
    let build_dir = FunctionBuildDir::PlatformSync(project_test.project.clone(), lambda_fn);
    let built_path = SwcBuilder::new(project_test.project.clone())
        .build(source_file, &build_dir)
        .unwrap();
    assert!(matches!(built_path.kind, SourceKind::FunctionBuild(_)));
    assert_eq!(
        built_path.rel.to_string_lossy().as_ref(),
        "routes/data/lambda.js"
    );
    assert_eq!(
        built_path.abs,
        project_test
            .project_dir
            .join(".l3/aws/API_ID/l3-PROJECT_NAME-data-get/debug/routes/data/lambda.js")
    );
    assert!(built_path.abs.is_file());
    assert_eq!(
        fs::read_to_string(built_path.abs).unwrap(),
        "export const GET = (e)=>{};\n"
    );
}

#[tokio::test]
async fn test_compile_ts_for_development() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.ts")
                .content("export const GET = (e: string) => {}"),
        )
        .build();
    let path = project_test.path("routes/data/lambda.ts");
    let code = fs::read_to_string(&path).unwrap();
    let expected = "export const GET = (e)=>{};\n";
    assert_eq!(compile_ts(code, &BuildMode::Debug).unwrap(), expected);
    assert_eq!(compile_ts_file(&path, &BuildMode::Debug).unwrap(), expected);
}

#[tokio::test]
async fn test_compile_ts_for_production() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.ts")
                .content("export const GET = (e: string) => {}"),
        )
        .build();
    let path = project_test.path("routes/data/lambda.ts");
    let code = fs::read_to_string(&path).unwrap();
    let expected = "export const GET=e=>{};";
    assert_eq!(compile_ts(code, &BuildMode::Release).unwrap(), expected);
    assert_eq!(
        compile_ts_file(&path, &BuildMode::Release).unwrap(),
        expected
    );
}

#[tokio::test]
async fn test_compile_ts_for_production_multi_module_snafu() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("src/data.ts")
                .content("export function makeData(): string { return 'DATA' }"),
        )
        .with_source(
            TestSource::with_path("src/print.ts")
                .content("import {makeData} from './data.js'\nconsole.log(makeData())"),
        )
        .build();
    let path = project_test.path("src/print.ts");
    let expected = "import{makeData}from\"./data.js\";console.log(makeData());";
    assert_eq!(
        compile_ts(fs::read_to_string(&path).unwrap(), &BuildMode::Release).unwrap(),
        expected
    );
    assert_eq!(
        compile_ts_file(&path, &BuildMode::Release).unwrap(),
        expected
    );
}

#[tokio::test]
async fn test_minify_js() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.ts")
                .content("export const GET = (xyz) => xyz * 2"),
        )
        .build();
    let path = project_test.path("routes/data/lambda.ts");
    let code = fs::read_to_string(&path).unwrap();
    let expected = "export const GET=o=>2*o;";
    assert_eq!(minify_js(code).unwrap(), expected);
    assert_eq!(minify_js_file(&path).unwrap(), expected);
}
