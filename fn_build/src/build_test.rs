use crate::paths::collect_files;
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
            handler_fn_name: "DELETE".to_string(),
            mode: BuildMode::Debug,
            output: FnOutputConfig {
                build_root: build_dir.path().to_path_buf(),
                create_archive: true,
            },
            project_dir: Arc::new(
                env::current_dir()
                    .unwrap()
                    .join("fixtures/node/js/http_route"),
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
async fn build_fn_produces_archive() {
    let fixture_path = "fixtures/node/js/npm_dependencies/with_subpath";
    let build_root_temp = TempDir::new().unwrap();
    let project_dir = Arc::new(env::current_dir().unwrap().join(fixture_path));
    let build_manifest = build_node_fn(FnBuildSpec {
        entrypoint: PathBuf::from("routes/data/lambda.js"),
        handler_fn_name: "GET".to_string(),
        mode: BuildMode::Debug,
        output: FnOutputConfig {
            build_root: build_root_temp.path().to_path_buf(),
            create_archive: true,
        },
        project_dir: project_dir.clone(),
        runtime: Runtime::Node(Arc::new(
            NodeConfig::read_node_config(&project_dir).unwrap(),
        )),
    })
    .await
    .unwrap();
    assert!(build_manifest.output.archive_file.is_some());
    let unzipped_root = build_root_temp.child("test_output");
    unzip(&build_manifest.output.archive_file.unwrap(), &unzipped_root);
    let build_dir = build_root_temp.child("debug");
    for source in &build_manifest.sources {
        assert_eq!(
            fs::read_to_string(build_dir.join(&source.path)).unwrap(),
            fs::read_to_string(unzipped_root.join(&source.path)).unwrap(),
            "unzipped fn source {} did not match build output in {}",
            source.path.to_string_lossy(),
            build_dir.to_string_lossy(),
        );
    }

    assert!(unzipped_root.join("node_modules").is_dir());
    let dependency_source_paths: Vec<PathBuf> = collect_files(&project_dir.join("node_modules"))
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
