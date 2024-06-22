use crate::code::env::EnvVarSources;
use crate::code::source::SourceFile;
use std::fs;
use std::path::PathBuf;
use temp_dir::TempDir;

use crate::lambda::{HttpMethod, LambdaFn, RouteKey};

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
    let project_dir = TempDir::new().unwrap();
    fs::create_dir_all(project_dir.path().join("routes/data")).unwrap();
    let source_file_path = PathBuf::from("routes/data/lambda.js");
    fs::write(project_dir.path().join(&source_file_path), "").unwrap();
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let lambda_fn = LambdaFn::new(
        EnvVarSources::new(Vec::new(), route_key.clone()).unwrap(),
        "GET".to_string(),
        &"my_proj".to_string(),
        route_key,
        SourceFile::create(source_file_path, project_dir.path().to_path_buf()).unwrap(),
    );
    assert_eq!(
        lambda_fn.fn_name(&"DEPLOY".to_string()),
        "l3-my_proj-data-get-DEPLOY".to_string()
    );
}

#[test]
fn test_lambda_fn_handler_path() {
    let project_dir = TempDir::new().unwrap();
    fs::create_dir_all(project_dir.path().join("routes/data")).unwrap();
    let source_file_path = PathBuf::from("routes/data/lambda.js");
    fs::write(project_dir.path().join(&source_file_path), "").unwrap();
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let lambda_fn = LambdaFn::new(
        EnvVarSources::new(Vec::new(), route_key.clone()).unwrap(),
        "GET".to_string(),
        &"my_proj".to_string(),
        route_key,
        SourceFile::create(source_file_path, project_dir.path().to_path_buf()).unwrap(),
    );
    assert_eq!(
        lambda_fn.handler_path(),
        "routes/data/lambda.GET".to_string()
    );
}
