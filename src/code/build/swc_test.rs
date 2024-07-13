use std::fs;

use crate::code::build::swc::{compile_ts, compile_ts_file, minify_js, minify_js_file, SwcBuilder};
use crate::code::build::Builder;
use crate::code::build::{BuildMode, BuildOptions};
use crate::code::source::path::SourceKind;
use crate::code::source::FunctionBuildDir;
use crate::testing::{ProjectTest, TestSource};

#[test]
fn test_swc_builder_development_mode_returns_original_source_path() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export const GET = (e) => {}"),
        )
        .build();
    let source_file = &project_test.source_file("routes/data/lambda.js");
    let build_dir = FunctionBuildDir::new("API".to_string(), BuildMode::Debug, "FN".to_string());
    let build_options = BuildOptions::new(
        build_dir,
        BuildMode::Debug,
        project_test.project_dir.clone(),
    );
    let result = SwcBuilder::new()
        .build(source_file, &build_options)
        .unwrap();
    assert!(result.kind == SourceKind::OriginalSource);
    assert_eq!(
        result.rel.to_string_lossy().as_ref(),
        "routes/data/lambda.js"
    );
}

#[test]
fn test_swc_builder_development_mode_writes_ts_to_build_dir() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.ts")
                .content("export const GET = (e: string) => {}"),
        )
        .build();
    let source_file = &project_test.source_file("routes/data/lambda.ts");
    let build_dir = FunctionBuildDir::new("API".to_string(), BuildMode::Debug, "FN".to_string());
    let build_options = BuildOptions::new(
        build_dir,
        BuildMode::Debug,
        project_test.project_dir.clone(),
    );
    let built_path = SwcBuilder::new()
        .build(source_file, &build_options)
        .unwrap();
    assert_eq!(
        built_path.rel.to_string_lossy().as_ref(),
        "routes/data/lambda.js"
    );
    assert!(built_path
        .abs
        .to_string_lossy()
        .as_ref()
        .ends_with(".l3/API/FN/dev/routes/data/lambda.js"));
    assert!(built_path.abs.is_file());
    assert_eq!(
        fs::read_to_string(built_path.abs).unwrap(),
        "export const GET = (e)=>{};\n"
    );
}

#[test]
fn test_compile_ts_for_development() {
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

#[test]
fn test_compile_ts_for_production() {
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

#[test]
fn test_compile_ts_for_production_multi_module_snafu() {
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

#[test]
fn test_minify_js() {
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
