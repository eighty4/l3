use crate::runtime::Runtime;
use crate::{
    parse_entrypoint, parse_fn, FnEntrypoint, FnHandler, FnRouting, HttpMethod, HttpRoute,
};
use crate::{FnParseError, FnParseSpec};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::test]
async fn parse_fn_errors_for_invalid_extension() {
    for entrypoint in &["README", "README.md"] {
        let parse_spec = FnParseSpec {
            entrypoint: PathBuf::from(entrypoint),
            project_dir: Arc::new(
                env::current_dir()
                    .unwrap()
                    .join("fixtures/node/js/http_routes/delete_fn"),
            ),
            runtime: Runtime::Node(Default::default()),
        };
        match parse_fn(parse_spec).await {
            Err(FnParseError::InvalidFileType) => {}
            _ => panic!(),
        };
    }
}

#[tokio::test]
async fn parse_entrypoint_of_js_fn() {
    assert_eq!(
        parse_entrypoint(FnParseSpec {
            entrypoint: PathBuf::from("routes/data/lambda.js"),
            project_dir: Arc::new(
                env::current_dir()
                    .unwrap()
                    .join("fixtures/node/js/http_routes/get_fn"),
            ),
            runtime: Runtime::Node(Default::default()),
        })
        .await
        .unwrap(),
        FnEntrypoint {
            handlers: vec!(FnHandler {
                fn_name: "GET".to_string(),
                routing: FnRouting::HttpRoute(HttpRoute {
                    method: HttpMethod::Get,
                    path: "data".to_string()
                })
            }),
            path: PathBuf::from("routes/data/lambda.js")
        }
    );
}

#[tokio::test]
async fn parse_entrypoint_of_python_fn() {
    assert_eq!(
        parse_entrypoint(FnParseSpec {
            entrypoint: PathBuf::from("routes/data/lambda.py"),
            project_dir: Arc::new(
                env::current_dir()
                    .unwrap()
                    .join("fixtures/python/http_routes/get_fn"),
            ),
            runtime: Runtime::Node(Default::default()),
        })
        .await
        .unwrap(),
        FnEntrypoint {
            handlers: vec!(FnHandler {
                fn_name: "get".to_string(),
                routing: FnRouting::HttpRoute(HttpRoute {
                    method: HttpMethod::Get,
                    path: "data".to_string()
                })
            }),
            path: PathBuf::from("routes/data/lambda.py")
        }
    );
}
