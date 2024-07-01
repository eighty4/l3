use std::fs;
use std::path::PathBuf;

use crate::code::checksum::ChecksumTree;
use crate::code::env::EnvVarSources;
use crate::lambda::{HttpMethod, RouteKey};
use crate::testing::{ProjectTest, SourceType, TestSource};

#[test]
fn test_checksum_tree_do_checksums_match_clean_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(SourceType::JavaScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_clean_checksum(),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    assert!(checksum_tree
        .do_checksums_match(&PathBuf::from("routes/data/lambda.js"))
        .unwrap());
}

#[test]
fn test_checksum_tree_do_checksums_match_dirty_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(SourceType::ModuleJavaScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_dirty_checksum(),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    assert!(!checksum_tree
        .do_checksums_match(&PathBuf::from("routes/data/lambda.mjs"))
        .unwrap());
}

#[test]
fn test_checksum_tree_do_checksums_match_without_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(SourceType::TypeScript, route_key.clone())
                .content("export const GET = () => {}")
                .without_checksum(),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    assert!(!checksum_tree
        .do_checksums_match(&PathBuf::from("routes/data/lambda.ts"))
        .unwrap());
}

#[test]
fn test_checksum_tree_do_all_checksums_match_clean_checksums() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(SourceType::JavaScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_clean_checksum(),
        )
        .with_source(
            TestSource::with_path("src/foo.js")
                .content("export const sayFoo = () => console.log('foo')")
                .with_clean_checksum_for_fn(route_key.clone()),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    assert!(checksum_tree
        .do_all_checksums_match(&project_test.source_paths())
        .unwrap());
}

#[test]
fn test_checksum_tree_do_all_checksums_match_dirty_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(SourceType::Python, route_key.clone())
                .content("export const GET = () => {}")
                .with_clean_checksum(),
        )
        .with_source(
            TestSource::with_path("src/foo.js")
                .content("export const sayFoo = () => console.log('foo')")
                .with_dirty_checksum_for_fn(route_key.clone()),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    assert!(!checksum_tree
        .do_all_checksums_match(&project_test.source_paths())
        .unwrap());
}

#[test]
fn test_checksum_tree_do_all_checksums_match_without_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(SourceType::JavaScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_clean_checksum(),
        )
        .with_source(
            TestSource::with_path("src/foo.js")
                .content("export const sayFoo = () => console.log('foo')")
                .without_checksum(),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    assert!(!checksum_tree
        .do_all_checksums_match(&project_test.source_paths())
        .unwrap());
}

#[test]
fn test_checksum_tree_do_env_checksums_match_with_clean_method_env_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::method_env_var(route_key.clone())
                .content("KEY=VALUE")
                .with_clean_checksum(),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    let env_var_sources = EnvVarSources::new(&project_test.project_dir, &route_key).unwrap();

    assert!(checksum_tree
        .do_env_checksums_match(&env_var_sources)
        .unwrap());
}

#[test]
fn test_checksum_tree_do_env_checksums_match_with_clean_path_env_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::path_env_var(route_key.clone())
                .content("KEY=VALUE")
                .with_clean_checksum(),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    let env_var_sources = EnvVarSources::new(&project_test.project_dir, &route_key).unwrap();

    assert!(checksum_tree
        .do_env_checksums_match(&env_var_sources)
        .unwrap());
}

#[test]
fn test_checksum_tree_do_env_checksums_match_with_dirty_method_env_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::method_env_var(route_key.clone())
                .content("KEY=VALUE")
                .with_dirty_checksum(),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    let env_var_sources = EnvVarSources::new(&project_test.project_dir, &route_key).unwrap();

    assert!(!checksum_tree
        .do_env_checksums_match(&env_var_sources)
        .unwrap());
}

#[test]
fn test_checksum_tree_do_env_checksums_match_with_dirty_path_env_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::path_env_var(route_key.clone())
                .content("KEY=VALUE")
                .with_dirty_checksum(),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    let env_var_sources = EnvVarSources::new(&project_test.project_dir, &route_key).unwrap();

    assert!(!checksum_tree
        .do_env_checksums_match(&env_var_sources)
        .unwrap());
}

#[test]
fn test_checksum_tree_do_env_checksums_match_with_method_env_checksum_without_source() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::method_env_var(route_key.clone())
                .content("KEY=VALUE")
                .with_clean_checksum(),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    fs::remove_file(
        &project_test
            .project_dir
            .join(route_key.to_route_dir_path())
            .join("lambda.get.env"),
    )
    .unwrap();

    let env_var_sources = EnvVarSources::new(&project_test.project_dir, &route_key).unwrap();

    assert!(!checksum_tree
        .do_env_checksums_match(&env_var_sources)
        .unwrap());
}

#[test]
fn test_checksum_tree_do_env_checksums_match_with_path_env_checksum_without_source() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::path_env_var(route_key.clone())
                .content("KEY=VALUE")
                .with_clean_checksum(),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    fs::remove_file(
        &project_test
            .project_dir
            .join(route_key.to_route_dir_path())
            .join("lambda.env"),
    )
    .unwrap();

    let env_var_sources = EnvVarSources::new(&project_test.project_dir, &route_key).unwrap();

    assert!(!checksum_tree
        .do_env_checksums_match(&env_var_sources)
        .unwrap());
}

#[test]
fn test_checksum_tree_update_env_var_checksums_updates_checksums() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::method_env_var(route_key.clone())
                .content("KEY=VALUE")
                .with_dirty_checksum(),
        )
        .with_source(
            TestSource::path_env_var(route_key.clone())
                .content("KEY=VALUE")
                .with_dirty_checksum(),
        )
        .build();

    let mut checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    let env_var_sources = EnvVarSources::new(&project_test.project_dir, &route_key).unwrap();
    assert!(!checksum_tree
        .do_env_checksums_match(&env_var_sources)
        .unwrap());
    checksum_tree
        .update_env_var_checksums(&env_var_sources)
        .unwrap();
    assert!(checksum_tree
        .do_env_checksums_match(&env_var_sources)
        .unwrap());
}

#[test]
fn test_checksum_tree_update_env_var_checksums_removes_checksums() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::method_env_var(route_key.clone())
                .content("KEY=VALUE")
                .with_dirty_checksum(),
        )
        .with_source(
            TestSource::path_env_var(route_key.clone())
                .content("KEY=VALUE")
                .with_dirty_checksum(),
        )
        .build();

    let mut checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    let env_var_sources = EnvVarSources::new(&project_test.project_dir, &route_key).unwrap();
    assert!(!checksum_tree
        .do_env_checksums_match(&env_var_sources)
        .unwrap());

    fs::remove_file(
        &project_test
            .project_dir
            .join(route_key.to_route_dir_path())
            .join("lambda.env"),
    )
    .unwrap();

    let env_var_sources = EnvVarSources::new(&project_test.project_dir, &route_key).unwrap();
    assert!(!checksum_tree
        .do_env_checksums_match(&env_var_sources)
        .unwrap());
    checksum_tree
        .update_env_var_checksums(&env_var_sources)
        .unwrap();
    assert!(checksum_tree
        .do_env_checksums_match(&env_var_sources)
        .unwrap());
    assert!(!&project_test
        .project_dir
        .join(route_key.to_route_dir_path())
        .join("lambda.env")
        .exists());
    assert!(!project_test
        .lambda_checksum_path(&route_key)
        .join("lambda.env")
        .exists());
}

#[test]
fn test_checksum_tree_remove_checksum_removes_from_cache_and_fs() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(SourceType::JavaScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_clean_checksum(),
        )
        .build();

    let mut checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();

    let p = PathBuf::from("routes/data/lambda.js");
    assert!(checksum_tree.do_checksums_match(&p).unwrap());
    checksum_tree.remove_checksum(&p);
    assert!(!checksum_tree.do_checksums_match(&p).unwrap());
    assert!(!p.exists());
}

#[test]
fn test_checksum_tree_update_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(SourceType::JavaScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_dirty_checksum(),
        )
        .build();

    let mut checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();
    let p = PathBuf::from("routes/data/lambda.js");
    assert!(!checksum_tree.do_checksums_match(&p).unwrap());
    checksum_tree.update_checksum(p.clone()).unwrap();
    assert!(checksum_tree.do_checksums_match(&p).unwrap());
}

#[test]
fn test_checksum_tree_update_all_checksums() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(SourceType::JavaScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_dirty_checksum(),
        )
        .with_source(
            TestSource::http_fn(SourceType::TypeScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_dirty_checksum(),
        )
        .build();

    let mut checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .unwrap();
    let p = vec![
        PathBuf::from("routes/data/lambda.js"),
        PathBuf::from("routes/data/lambda.ts"),
    ];
    assert!(!checksum_tree.do_all_checksums_match(&p).unwrap());
    checksum_tree.update_all_checksums(p.clone()).unwrap();
    assert!(checksum_tree.do_all_checksums_match(&p).unwrap());
}
