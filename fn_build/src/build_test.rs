use crate::runtime::node::{build_node_fn, NodeConfig};
use crate::runtime::Runtime;
use crate::testing::unzip;
use crate::{build_fn, FnOutputConfig};
use crate::{BuildMode, FnBuildSpec};
use crate::{FnBuildError, FnParseError};
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs};
use temp_dir::TempDir;

#[tokio::test]
async fn build_fn_errors_for_invalid_extension() {
    let build_dir = TempDir::new().unwrap();
    for entrypoint in &["README", "README.md"] {
        let build_spec = FnBuildSpec {
            entrypoint: PathBuf::from(entrypoint),
            handler_fn_name: "GET".to_string(),
            mode: BuildMode::Debug,
            output: FnOutputConfig {
                build_root: build_dir.path().to_path_buf(),
                create_archive: true,
                use_build_mode: true,
            },
            project_dir: Arc::new(
                env::current_dir()
                    .unwrap()
                    .join("fixtures/node/js/http_routes/get_fn"),
            ),
            runtime: Runtime::Node(Default::default()),
        };
        match build_fn(build_spec).await {
            Err(FnBuildError::ParseError(FnParseError::InvalidFileType)) => {}
            _ => panic!(),
        };
    }
}

#[tokio::test]
async fn build_fn_errors_for_missing_handler() {
    let fixture_path = "fixtures/node/js/npm_dependencies/with_subpath";
    let build_root_temp = TempDir::new().unwrap();
    let project_dir = Arc::new(env::current_dir().unwrap().join(fixture_path));
    let result = build_node_fn(FnBuildSpec {
        entrypoint: PathBuf::from("routes/data/lambda.js"),
        handler_fn_name: "someWackyFunctionName".to_string(),
        mode: BuildMode::Debug,
        output: FnOutputConfig {
            build_root: build_root_temp.path().to_path_buf(),
            create_archive: true,
            use_build_mode: true,
        },
        project_dir: project_dir.clone(),
        runtime: Runtime::Node(Some(Arc::new(
            NodeConfig::read_node_config(&project_dir).unwrap(),
        ))),
    })
    .await;
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().to_string(),
        "error parsing function: entrypoint routes/data/lambda.js does not have a handler fn someWackyFunctionName"
    );
}

#[tokio::test]
async fn build_fn_produces_archive() {
    let fixture_path = "fixtures/node/js/npm_dependencies/with_subpath";
    let build_root_temp = TempDir::new().unwrap();
    let project_dir = Arc::new(env::current_dir().unwrap().join(fixture_path));
    let build_manifest = build_node_fn(FnBuildSpec {
        entrypoint: PathBuf::from("routes/data/lambda.js"),
        handler_fn_name: "DELETE".to_string(),
        mode: BuildMode::Debug,
        output: FnOutputConfig {
            build_root: build_root_temp.path().to_path_buf(),
            create_archive: true,
            use_build_mode: true,
        },
        project_dir: project_dir.clone(),
        runtime: Runtime::Node(Some(Arc::new(
            NodeConfig::read_node_config(&project_dir).unwrap(),
        ))),
    })
    .await
    .unwrap();
    assert!(build_manifest.output.archive_file.is_some());
    let unzipped_root = build_root_temp.child("test_output");
    unzip(&build_manifest.output.archive_file.unwrap(), &unzipped_root);
    for source in &build_manifest.sources {
        assert_eq!(
            fs::read_to_string(build_manifest.output.build_dir.join(&source.path)).unwrap(),
            fs::read_to_string(unzipped_root.join(&source.path)).unwrap(),
            "unzipped fn source {} did not match build output in {}",
            source.path.to_string_lossy(),
            build_manifest.output.build_dir.to_string_lossy(),
        );
    }

    assert!(unzipped_root.join("node_modules").is_dir());
    let dependency_source_paths: Vec<PathBuf> =
        l3_api_base::collect_files(&project_dir.join("node_modules"))
            .iter()
            .map(|p| p.strip_prefix(project_dir.as_ref()).unwrap().to_path_buf())
            .collect();
    assert!(!dependency_source_paths.is_empty());
    for path in dependency_source_paths {
        assert_eq!(
            fs::read_to_string(project_dir.join(&path)).unwrap(),
            fs::read_to_string(unzipped_root.join(&path)).unwrap(),
        );
    }
}

#[tokio::test]
async fn build_fn_produces_checksums() {
    let fixture_path = "fixtures/node/js/npm_dependencies/with_subpath";
    let build_root_temp = TempDir::new().unwrap();
    let project_dir = Arc::new(env::current_dir().unwrap().join(fixture_path));
    let build_manifest = build_node_fn(FnBuildSpec {
        entrypoint: PathBuf::from("routes/data/lambda.js"),
        handler_fn_name: "DELETE".to_string(),
        mode: BuildMode::Debug,
        output: FnOutputConfig {
            build_root: build_root_temp.path().to_path_buf(),
            create_archive: false,
            use_build_mode: true,
        },
        project_dir: project_dir.clone(),
        runtime: Runtime::Node(Some(Arc::new(
            NodeConfig::read_node_config(&project_dir).unwrap(),
        ))),
    })
    .await
    .unwrap();
    assert!(build_manifest.output.archive_file.is_none());
    assert_eq!(build_manifest.checksums.iter().len(), 2);
    assert!(build_manifest
        .checksums
        .get(&PathBuf::from("package.json"))
        .is_some());
    assert!(build_manifest
        .checksums
        .get(&PathBuf::from("routes/data/lambda.js"))
        .is_some());
}
