use crate::runtime::Runtime;
use crate::testing::{create_node_fixture, unzip};
use crate::FnBuildError;
use crate::{build_fn, FnBuildOutput};
use crate::{BuildMode, FnBuildSpec, FnParseSpec};
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs};
use temp_dir::TempDir;

#[tokio::test]
async fn build_fn_errors_for_invalid_extension() {
    let build_dir = TempDir::new().unwrap();
    for entrypoint in &["README", "README.md"] {
        let build_spec = FnBuildSpec {
            function: FnParseSpec {
                entrypoint: PathBuf::from(entrypoint),
                project_dir: Arc::new(
                    env::current_dir()
                        .unwrap()
                        .join("fixtures/node/js/http_route"),
                ),
                runtime: Runtime::Node(Default::default()),
            },
            mode: BuildMode::Debug,
            output: FnBuildOutput::Directory(build_dir.path().to_path_buf()),
        };
        match build_fn(build_spec).await {
            Err(FnBuildError::InvalidFileType) => {}
            _ => panic!(),
        };
    }
}

#[tokio::test]
async fn build_fn_produces_archive() {
    let fixture_dir = PathBuf::from("fixtures/node/js/relative_import");
    let test_fixture = create_node_fixture(fixture_dir.clone());
    let archive_file = test_fixture.build_path(&PathBuf::from("archive.zip"));
    let result = test_fixture
        .build(
            BuildMode::Debug,
            FnBuildOutput::Archive {
                build_root: test_fixture.build_root(),
                archive_file: archive_file.clone(),
            },
        )
        .await
        .unwrap();
    let build_dir = test_fixture.build_output_dir(&BuildMode::Debug);
    let unzipped_root = test_fixture.build_path(&PathBuf::from("unzipped"));
    unzip(&archive_file, &unzipped_root);

    for source in &result.manifest.sources {
        assert_eq!(
            fs::read_to_string(build_dir.join(&source.path)).unwrap(),
            fs::read_to_string(unzipped_root.join(&source.path)).unwrap(),
            "unzipped fn source {} did not match build output in {}",
            source.path.to_string_lossy(),
            build_dir.to_string_lossy(),
        );
    }
}
