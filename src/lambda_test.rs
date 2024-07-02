use std::path::PathBuf;

use crate::code::env::EnvVarSources;
use crate::lambda::{HttpMethod, LambdaFn, RouteKey};
use crate::testing::{ProjectTest, TestSource};

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
fn test_route_key_try_from() {
    let route_key = RouteKey::try_from("GET /data".to_string()).unwrap();
    assert_eq!(route_key.http_method, HttpMethod::Get);
    assert_eq!(route_key.http_path, "data".to_string());
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

#[test]
fn test_lambda_fn_fn_name() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export function DELETE(){}"),
        )
        .build();
    let route_key = RouteKey::new(HttpMethod::Delete, "data".to_string());
    let lambda_fn = LambdaFn::new(
        EnvVarSources::new(&project_test.project_dir, &route_key).unwrap(),
        "DELETE".to_string(),
        PathBuf::from("routes/data/lambda.js"),
        &"my_proj".to_string(),
        route_key,
    );
    assert_eq!(lambda_fn.fn_name, "l3-my_proj-data-delete");
}

#[test]
fn test_lambda_fn_handler_path() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export function GET(){}"),
        )
        .build();
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let lambda_fn = LambdaFn::new(
        EnvVarSources::new(&project_test.project_dir, &route_key).unwrap(),
        "GET".to_string(),
        PathBuf::from("routes/data/lambda.js"),
        &"my_proj".to_string(),
        route_key,
    );
    assert_eq!(
        lambda_fn.handler_path(),
        "routes/data/lambda.GET".to_string()
    );
}
