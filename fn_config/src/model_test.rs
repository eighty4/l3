use crate::{HttpMethod, HttpRoute};
use std::path::PathBuf;

#[test]
fn test_http_method_from_str_ok() {
    let test_data = [
        ("GET", HttpMethod::Get),
        ("DELETE", HttpMethod::Delete),
        ("PATCH", HttpMethod::Patch),
        ("POST", HttpMethod::Post),
        ("Put", HttpMethod::Put),
    ];
    for (handler_fn_name, http_method) in test_data {
        assert_eq!(HttpMethod::try_from(handler_fn_name).unwrap(), http_method);
    }
}

#[test]
fn test_http_method_from_str_err() {
    if let Err(err) = HttpMethod::try_from("not much of a golfer") {
        assert_eq!(
            err.to_string(),
            "could not resolve http method from not much of a golfer".to_string()
        );
    } else {
        panic!();
    }
}

#[test]
fn test_http_route_new_ok() {
    let test_data = [
        (
            "GET",
            HttpMethod::Get,
            "routes/data/api/lambda.js",
            "data/api",
        ),
        (
            "DELETE",
            HttpMethod::Delete,
            "routes/data/lambda.js",
            "data",
        ),
        ("PATCH", HttpMethod::Patch, "routes/lambda.js", ""),
        ("POST", HttpMethod::Post, "routes/lambda.js", ""),
        ("Put", HttpMethod::Put, "routes/lambda.js", ""),
    ];
    for (handler_fn_name, http_method, source_path, http_path) in test_data {
        assert_eq!(
            HttpRoute::new(&PathBuf::from(source_path), handler_fn_name).unwrap(),
            HttpRoute {
                method: http_method,
                path: String::from(http_path),
            }
        );
    }
}

#[test]
fn test_http_route_new_err() {
    let bad_method_result = HttpRoute::new(&PathBuf::from("routes/lambda.js"), "UPDATE");
    assert!(bad_method_result.is_err());
    assert_eq!(
        bad_method_result.err().map(|err| err.to_string()),
        Some("could not resolve http method from UPDATE".to_string())
    );

    let bad_path_result = HttpRoute::new(&PathBuf::from("codes/lambda.js"), "put");
    assert!(bad_path_result.is_err());
    assert_eq!(
        bad_path_result.err().map(|err| err.to_string()),
        Some("not an http route source path".to_string())
    );
}
