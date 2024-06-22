use crate::code::env::{is_valid_env_file_name, parse_env_file_name_for_http_method};
use crate::lambda::HttpMethod;

#[test]
fn test_is_valid_env_file_name() {
    assert!(is_valid_env_file_name(".env"));
    assert!(is_valid_env_file_name("delete.env"));
    assert!(is_valid_env_file_name("get.env"));
    assert!(is_valid_env_file_name("patch.env"));
    assert!(is_valid_env_file_name("post.env"));
    assert!(is_valid_env_file_name("put.env"));
    assert!(!is_valid_env_file_name("gibberish.env"));
    assert!(!is_valid_env_file_name("extra.post.env"));
}

#[test]
fn test_parse_env_file_name_for_http_method() {
    assert!(parse_env_file_name_for_http_method(".env").is_none());
    assert_eq!(
        parse_env_file_name_for_http_method("delete.env").unwrap(),
        HttpMethod::Delete
    );
    assert_eq!(
        parse_env_file_name_for_http_method("get.env").unwrap(),
        HttpMethod::Get
    );
    assert_eq!(
        parse_env_file_name_for_http_method("patch.env").unwrap(),
        HttpMethod::Patch
    );
    assert_eq!(
        parse_env_file_name_for_http_method("post.env").unwrap(),
        HttpMethod::Post
    );
    assert_eq!(
        parse_env_file_name_for_http_method("put.env").unwrap(),
        HttpMethod::Put
    );
}
