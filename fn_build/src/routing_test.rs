use crate::{FnRouting, HttpMethod, HttpRoute};
use std::path::PathBuf;

#[test]
fn test_http_route_from_handler_fn() {
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
            FnRouting::from_handler_fn(&PathBuf::from(source_path), handler_fn_name),
            FnRouting::HttpRoute(HttpRoute {
                method: http_method,
                path: String::from(http_path),
            })
        );
    }
}

#[test]
fn test_unsupported_route_from_handler_fn() {
    assert_eq!(
        FnRouting::from_handler_fn(&PathBuf::from("routes/data/lambda.js"), "get_that_data"),
        FnRouting::Unsupported
    );
}
