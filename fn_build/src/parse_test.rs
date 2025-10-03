use temp_dir::TempDir;

use crate::runtime::Runtime;
use crate::{
    parse_entrypoint, parse_fn, FnEntrypoint, FnHandler, FnRouting, HttpMethod, HttpRoute,
};
use crate::{FnParseError, FnParseSpec};
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs};

#[tokio::test]
async fn parse_fn_errors_for_invalid_extension() {
    let temp_dir = TempDir::new().unwrap();
    for entrypoint in &["README", "README.md"] {
        fs::write(temp_dir.child(entrypoint), "").unwrap();
        let parse_spec = FnParseSpec {
            entrypoint: PathBuf::from(entrypoint),
            project_dir: Arc::new(temp_dir.path().to_path_buf()),
            runtime: Runtime::Node(Default::default()),
        };
        match parse_fn(parse_spec).await {
            Err(FnParseError::InvalidFileType) => {}
            _ => panic!(),
        };
    }
}

#[tokio::test]
async fn parse_fn_errors_for_missing_entrypoint() {
    let temp_dir = TempDir::new().unwrap();
    let entrypoint = "data.ts";
    let parse_spec = FnParseSpec {
        entrypoint: PathBuf::from(entrypoint),
        project_dir: Arc::new(temp_dir.path().to_path_buf()),
        runtime: Runtime::Node(Default::default()),
    };
    match parse_fn(parse_spec).await {
        Err(FnParseError::MissingEntrypoint(_)) => {}
        _ => panic!(),
    };
}

#[tokio::test]
async fn parse_entrypoint_errors_for_invalid_extension() {
    let temp_dir = TempDir::new().unwrap();
    for entrypoint in &["README", "README.md"] {
        fs::write(temp_dir.child(entrypoint), "").unwrap();
        let parse_spec = FnParseSpec {
            entrypoint: PathBuf::from(entrypoint),
            project_dir: Arc::new(temp_dir.path().to_path_buf()),
            runtime: Runtime::Node(Default::default()),
        };
        match parse_entrypoint(parse_spec).await {
            Err(FnParseError::InvalidFileType) => {}
            _ => panic!(),
        };
    }
}

#[tokio::test]
async fn parse_entrypoint_errors_for_missing_entrypoint() {
    let temp_dir = TempDir::new().unwrap();
    let entrypoint = "data.ts";
    let parse_spec = FnParseSpec {
        entrypoint: PathBuf::from(entrypoint),
        project_dir: Arc::new(temp_dir.path().to_path_buf()),
        runtime: Runtime::Node(Default::default()),
    };
    match parse_entrypoint(parse_spec).await {
        Err(FnParseError::MissingEntrypoint(_)) => {}
        _ => panic!(),
    };
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
