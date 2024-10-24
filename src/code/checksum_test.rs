use std::path::PathBuf;

use crate::code::checksum::ChecksumTree;
use crate::code::source::Language;
use crate::lambda::{HttpMethod, RouteKey};
use crate::testing::project::ProjectTest;
use crate::testing::source::TestSource;

#[tokio::test]
async fn test_checksum_tree_do_checksums_match_clean_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(Language::JavaScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_clean_checksum(),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .await
    .unwrap();

    assert!(checksum_tree
        .do_checksums_match(&PathBuf::from("routes/data/lambda.js"))
        .unwrap());
}

#[tokio::test]
async fn test_checksum_tree_do_checksums_match_dirty_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(Language::JavaScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_dirty_checksum(),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .await
    .unwrap();

    assert!(!checksum_tree
        .do_checksums_match(&PathBuf::from("routes/data/lambda.js"))
        .unwrap());
}

#[tokio::test]
async fn test_checksum_tree_do_checksums_match_without_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(Language::TypeScript, route_key.clone())
                .content("export const GET = () => {}")
                .without_checksum(),
        )
        .build();

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .await
    .unwrap();

    assert!(!checksum_tree
        .do_checksums_match(&PathBuf::from("routes/data/lambda.ts"))
        .unwrap());
}

#[tokio::test]
async fn test_checksum_tree_do_all_checksums_match_clean_checksums() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(Language::JavaScript, route_key.clone())
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
    .await
    .unwrap();

    assert!(checksum_tree
        .do_all_checksums_match(
            &project_test
                .source_paths()
                .iter()
                .map(|p| p.rel().clone())
                .collect()
        )
        .unwrap());
}

#[tokio::test]
async fn test_checksum_tree_do_all_checksums_match_dirty_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(Language::Python, route_key.clone())
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
    .await
    .unwrap();

    assert!(!checksum_tree
        .do_all_checksums_match(
            &project_test
                .source_paths()
                .iter()
                .map(|p| p.rel().clone())
                .collect()
        )
        .unwrap());
}

#[tokio::test]
async fn test_checksum_tree_do_all_checksums_match_without_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(Language::JavaScript, route_key.clone())
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
    .await
    .unwrap();

    assert!(!checksum_tree
        .do_all_checksums_match(
            &project_test
                .source_paths()
                .iter()
                .map(|p| p.rel().clone())
                .collect()
        )
        .unwrap());
}

#[tokio::test]
async fn test_checksum_tree_remove_checksum_removes_from_cache_and_fs() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(Language::JavaScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_clean_checksum(),
        )
        .build();

    let mut checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .await
    .unwrap();

    let p = PathBuf::from("routes/data/lambda.js");
    assert!(checksum_tree.do_checksums_match(&p).unwrap());
    checksum_tree.remove_checksum(&p);
    assert!(!checksum_tree.do_checksums_match(&p).unwrap());
    assert!(!p.exists());
}

#[tokio::test]
async fn test_checksum_tree_update_checksum() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(Language::JavaScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_dirty_checksum(),
        )
        .build();

    let mut checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .await
    .unwrap();
    let p = PathBuf::from("routes/data/lambda.js");
    assert!(!checksum_tree.do_checksums_match(&p).unwrap());
    checksum_tree.update_checksum(p.clone()).unwrap();
    assert!(checksum_tree.do_checksums_match(&p).unwrap());
}

#[tokio::test]
async fn test_checksum_tree_update_all_checksums() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME")
        .with_source(
            TestSource::http_fn(Language::JavaScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_dirty_checksum(),
        )
        .with_source(
            TestSource::http_fn(Language::TypeScript, route_key.clone())
                .content("export const GET = () => {}")
                .with_dirty_checksum(),
        )
        .build();

    let mut checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .await
    .unwrap();
    let p = vec![
        PathBuf::from("routes/data/lambda.js"),
        PathBuf::from("routes/data/lambda.ts"),
    ];
    assert!(!checksum_tree.do_all_checksums_match(&p).unwrap());
    let src_p = vec![
        project_test.source_path("routes/data/lambda.js"),
        project_test.source_path("routes/data/lambda.ts"),
    ];
    checksum_tree.update_all_checksums(&src_p).unwrap();
    assert!(checksum_tree.do_all_checksums_match(&p).unwrap());
}
