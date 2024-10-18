use std::path::PathBuf;

use crate::lambda::{HttpMethod, LambdaFn, RouteKey};
use crate::testing::project::ProjectTest;
use crate::testing::source::TestSource;

#[test]
fn test_route_key_extract_http_path_not_a_routes_dir_path() {
    assert!(RouteKey::extract_http_path(&PathBuf::from("src/data.js")).is_none());
}

#[test]
fn test_route_key_extract_http_path() {
    let http_path = RouteKey::extract_http_path(&PathBuf::from("routes/data/lambda.js"));
    assert_eq!(http_path.unwrap().as_str(), "data");
}

#[test]
fn test_route_key_new() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    assert_eq!(route_key.http_method, HttpMethod::Get);
    assert_eq!(route_key.http_path, "data");
}

#[test]
fn test_route_key_new_removes_slash_prefix() {
    let route_key = RouteKey::new(HttpMethod::Get, "/data".to_string());
    assert_eq!(route_key.http_path, "data");
}

#[test]
fn test_route_key_to_route_key() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    assert_eq!(route_key.to_route_key_string(), "GET /data");
}

#[test]
fn test_route_key_to_route_dir_path() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    assert_eq!(
        route_key.to_route_dir_path(),
        PathBuf::from("routes").join("data")
    );
}

#[test]
fn test_route_key_from_route_key_string() {
    assert!(
        RouteKey::from_route_key_string("GET /data".to_string()).unwrap()
            == RouteKey::new(HttpMethod::Get, "data".to_string())
    );
    assert!(
        RouteKey::from_route_key_string("POST data".to_string()).unwrap()
            == RouteKey::new(HttpMethod::Post, "data".to_string())
    );
}

#[test]
fn test_http_method_try_from() {
    assert_eq!(HttpMethod::try_from("delete").unwrap(), HttpMethod::Delete);
    assert_eq!(HttpMethod::try_from("DELETE").unwrap(), HttpMethod::Delete);
    assert_eq!(HttpMethod::try_from("get").unwrap(), HttpMethod::Get);
    assert_eq!(HttpMethod::try_from("GET").unwrap(), HttpMethod::Get);
    assert_eq!(HttpMethod::try_from("patch").unwrap(), HttpMethod::Patch);
    assert_eq!(HttpMethod::try_from("PATCH").unwrap(), HttpMethod::Patch);
    assert_eq!(HttpMethod::try_from("post").unwrap(), HttpMethod::Post);
    assert_eq!(HttpMethod::try_from("POST").unwrap(), HttpMethod::Post);
    assert_eq!(HttpMethod::try_from("put").unwrap(), HttpMethod::Put);
    assert_eq!(HttpMethod::try_from("PUT").unwrap(), HttpMethod::Put);
}

#[tokio::test]
async fn test_lambda_fn_fn_name() {
    let project_test = ProjectTest::builder()
        .project_name("my_proj")
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export function DELETE(){}"),
        )
        .build();
    let route_key = RouteKey::new(HttpMethod::Delete, "data".to_string());
    let lambda_fn = LambdaFn::new(
        "DELETE".to_string(),
        project_test.source_path("routes/data/lambda.js"),
        project_test.project.clone(),
        route_key,
    );
    assert_eq!(lambda_fn.fn_name, "l3-my_proj-data-delete");
}

#[tokio::test]
async fn test_lambda_fn_handler_path() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export function GET(){}"),
        )
        .build();
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let lambda_fn = LambdaFn::new(
        "GET".to_string(),
        project_test.source_path("routes/data/lambda.js"),
        project_test.project.clone(),
        route_key,
    );
    assert_eq!(
        lambda_fn.handler_path(),
        "routes/data/lambda.GET".to_string()
    );
}
