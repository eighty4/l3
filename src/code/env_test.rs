use crate::code::env::{
    is_valid_env_file_name, parse_env_file_name_for_http_method, EnvVarSources,
};
use crate::lambda::{HttpMethod, RouteKey};
use crate::testing::project::ProjectTest;
use crate::testing::source::TestSource;

#[test]
fn test_is_valid_env_file_name() {
    assert!(is_valid_env_file_name("lambda.env"));
    assert!(is_valid_env_file_name("lambda.delete.env"));
    assert!(is_valid_env_file_name("lambda.get.env"));
    assert!(is_valid_env_file_name("lambda.patch.env"));
    assert!(is_valid_env_file_name("lambda.post.env"));
    assert!(is_valid_env_file_name("lambda.put.env"));
    assert!(!is_valid_env_file_name("gibberish.env"));
    assert!(!is_valid_env_file_name("lambda.gibberish.env"));
    assert!(!is_valid_env_file_name("extra.post.env"));
}

#[test]
fn test_parse_env_file_name_for_http_method() {
    assert!(parse_env_file_name_for_http_method("lambda.env").is_none());
    assert_eq!(
        parse_env_file_name_for_http_method("lambda.delete.env").unwrap(),
        HttpMethod::Delete
    );
    assert_eq!(
        parse_env_file_name_for_http_method("lambda.get.env").unwrap(),
        HttpMethod::Get
    );
    assert_eq!(
        parse_env_file_name_for_http_method("lambda.patch.env").unwrap(),
        HttpMethod::Patch
    );
    assert_eq!(
        parse_env_file_name_for_http_method("lambda.post.env").unwrap(),
        HttpMethod::Post
    );
    assert_eq!(
        parse_env_file_name_for_http_method("lambda.put.env").unwrap(),
        HttpMethod::Put
    );
}

#[tokio::test]
async fn test_env_var_sources_read_method_vars() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .with_source(TestSource::method_env_var(route_key.clone()).content("GREETING=yo"))
        .build();
    let env_var_sources = EnvVarSources::new(&project_test.project_dir, &route_key).unwrap();
    let env_vars = env_var_sources.read_env_variables().unwrap();
    assert!(env_vars.is_some());
    assert_eq!(env_vars.unwrap().get("GREETING").unwrap().as_str(), "yo");
}

#[tokio::test]
async fn test_env_var_sources_read_path_vars() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .with_source(TestSource::path_env_var(route_key.clone()).content("GREETING=sup"))
        .build();
    let env_var_sources = EnvVarSources::new(&project_test.project_dir, &route_key).unwrap();
    let env_vars = env_var_sources.read_env_variables().unwrap();
    assert!(env_vars.is_some());
    assert_eq!(env_vars.unwrap().get("GREETING").unwrap().as_str(), "sup");
}

#[tokio::test]
async fn test_env_var_sources_read_method_overrides_path_vars() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .with_source(TestSource::method_env_var(route_key.clone()).content("GREETING=yo"))
        .with_source(TestSource::path_env_var(route_key.clone()).content("GREETING=sup"))
        .build();
    let env_var_sources = EnvVarSources::new(&project_test.project_dir, &route_key).unwrap();
    let env_vars = env_var_sources.read_env_variables().unwrap();
    assert!(env_vars.is_some());
    assert_eq!(env_vars.unwrap().get("GREETING").unwrap().as_str(), "yo");
}
