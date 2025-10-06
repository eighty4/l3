use crate::testing::fixture::TestFixtureSpec;
use crate::testing::result::FixtureError;
use crate::testing::scenarios::BuildVariation;
use crate::testing::verify_runtime::verify_with_runtime;
use crate::{build_fn, BuildMode, FnBuildSpec, FnOutputConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use temp_dir::TempDir;

pub async fn verify_build(
    spec: &Arc<TestFixtureSpec>,
    mode: &BuildMode,
    variation: &Option<BuildVariation>,
) -> Result<(), FixtureError> {
    let temp_dir = TempDir::new().unwrap();
    let build_manifest = match build_fn(FnBuildSpec {
        entrypoint: spec.entrypoint.to_path_buf(),
        handler_fn_name: spec.handler_fn_name.to_string(),
        mode: mode.clone(),
        output: FnOutputConfig {
            build_root: temp_dir.path().to_path_buf(),
            create_archive: false,
            dirname: "build-fixture-test".into(),
            use_build_mode: true,
        },
        project_dir: spec.fixture_dir.clone(),
        runtime: spec.fixture_runtime(),
    })
    .await
    {
        Ok(result) => result,
        Err(build_error) => return Err(FixtureError::Build(build_error)),
    };
    let build_dir = temp_dir
        .child(match &mode {
            BuildMode::Debug => "debug",
            BuildMode::Release => "release",
        })
        .join("build-fixture-test");
    expect_build_result(spec, &build_dir, mode);
    verify_with_runtime(
        spec,
        Some((build_dir, build_manifest)),
        variation
            .as_ref()
            .map(|BuildVariation::Node(variation)| variation.bin.to_path_buf()),
    )
}

fn expect_build_result(spec: &Arc<TestFixtureSpec>, build_dir: &Path, mode: &BuildMode) {
    let expected_build_result = BuildResult::read_json(&spec.fixture_dir, mode).unwrap();
    for expected_file in &expected_build_result.files {
        let built_file_path: PathBuf = match &expected_file.result.path {
            Some(path) => build_dir.join(path),
            None => build_dir.join(&expected_file.path),
        };
        let built_content = fs::read_to_string(&built_file_path).unwrap();
        let expected_content = match &expected_file.result.content {
            BuildFileContent::Transformed(expected_content) => expected_content,
            BuildFileContent::Identical => {
                let original_file_path = spec.fixture_dir.join(&expected_file.path);
                &fs::read_to_string(&original_file_path).unwrap()
            }
        };
        assert_eq!(
            built_content.replace("\r\n", "\n").as_str(),
            expected_content.replace("\r\n", "\n").as_str(),
            "{} from fixture {spec} did not match expected content in build dir {}",
            expected_file.path.to_string_lossy(),
            build_dir.to_string_lossy(),
        );
    }
}

/// Represents expected output from `fn_build`.
/// Deserialized from .fixture/debug_build.json or .fixture/release_build.json.
#[derive(Deserialize, Serialize)]
struct BuildResult {
    pub files: Vec<BuildFile>,
}

/// A source file in a build output directory.
#[derive(Deserialize, Serialize)]
struct BuildFile {
    pub path: PathBuf,
    pub result: BuildFileOutput,
}

#[derive(Deserialize, Serialize)]
struct BuildFileOutput {
    pub content: BuildFileContent,
    pub path: Option<PathBuf>,
}

/// Expected content for a build output file.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum BuildFileContent {
    Transformed(String),
    Identical,
}

impl BuildResult {
    pub fn read_json(fixture_dir: &Path, build_mode: &BuildMode) -> Option<BuildResult> {
        let filename = match build_mode {
            BuildMode::Debug => "build_debug.json",
            BuildMode::Release => "build_release.json",
        };
        let path = fixture_dir.join(".fixture").join(filename);
        if path.is_file() {
            Some(serde_json::from_str(fs::read_to_string(path).unwrap().as_str()).unwrap())
        } else {
            None
        }
    }
}
