use crate::testing::result::{BuildFileOutput, BuildResult};
use crate::testing::runtimes::{create_test_runtime, TestRuntime};
use crate::testing::spec::TestFixtureSpec;
use crate::{
    BuildMode, FnBuildManifest, FnBuildResult, FnBuildSpec, FnOutputConfig, FnParseManifest,
    FnParseSpec,
};
use anyhow::anyhow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{env, fs};
use temp_dir::TempDir;

pub struct TestFixture {
    /// Used for FnBuildOutput::Directory.
    build_root_temp: TempDir,
    /// TestRuntime integrates runtime specific features for test fixtures.
    runtime: Arc<Box<dyn TestRuntime>>,
    /// Fixture directory in //fn_build/fixtures.
    fixture_dir: Arc<PathBuf>,
    /// Model for test read from .fixture/spec.json.
    spec: TestFixtureSpec,
    gold_update: bool,
}

impl TestFixture {
    pub fn new(fixture_dir: PathBuf) -> Self {
        Self::new_inner(fixture_dir, false)
    }

    pub fn gold_update(fixture_dir: PathBuf) -> Self {
        Self::new_inner(fixture_dir, true)
    }

    fn new_inner(fixture_dir: PathBuf, gold_update: bool) -> Self {
        debug_assert!(fixture_dir.is_absolute());
        debug_assert!(fixture_dir.is_dir());
        let spec = TestFixtureSpec::try_from(&fixture_dir).unwrap();
        Self {
            build_root_temp: TempDir::new().unwrap(),
            runtime: create_test_runtime(&spec.entrypoint),
            fixture_dir: Arc::new(fixture_dir),
            spec,
            gold_update,
        }
    }

    pub async fn run(&self) {
        if self.gold_update {
            self.update_gold().await
        } else {
            // if not a typescript project, let's run fixture with runtime pre-build to verify fixture set up
            if !self.fixture_dir.join("tsconfig.json").exists() {
                self.verify_build_with_runtime(&self.fixture_dir, None, None)
                    .await;
            }
            let expected_manifest = self.verify_parse().await.unwrap();
            if let Some(debug_result) = BuildResult::read_json(&self.fixture_dir, &BuildMode::Debug)
            {
                self.run_build_test(BuildMode::Debug, debug_result, &expected_manifest)
                    .await;
            }
            if let Some(release_result) =
                BuildResult::read_json(&self.fixture_dir, &BuildMode::Release)
            {
                self.run_build_test(BuildMode::Release, release_result, &expected_manifest)
                    .await;
            }
        }
    }

    async fn update_gold(&self) {
        let parse_manifest = self
            .runtime
            .parse(FnParseSpec {
                entrypoint: self.spec.entrypoint.clone(),
                project_dir: self.fixture_dir.clone(),
                runtime: self.runtime.config(&self.fixture_dir),
            })
            .await
            .unwrap();
        let parse_json_path = self.fixture_dir.join(".fixture/parse.json");
        _ = fs::remove_file(&parse_json_path);
        fs::write(
            &parse_json_path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&parse_manifest).unwrap()
            ),
        )
        .unwrap();
    }

    async fn build(&self, mode: BuildMode) -> FnBuildResult<FnBuildManifest> {
        self.runtime
            .build(FnBuildSpec {
                entrypoint: self.spec.entrypoint.clone(),
                handler_fn_name: self.spec.handler_fn_name.clone(),
                mode,
                output: FnOutputConfig {
                    build_root: self.build_root_temp.path().to_path_buf(),
                    create_archive: false,
                    dirname: "build-fixture-test".into(),
                    use_build_mode: true,
                },
                project_dir: self.fixture_dir.clone(),
                runtime: self.runtime.config(&self.fixture_dir),
            })
            .await
    }

    async fn run_build_test(
        &self,
        mode: BuildMode,
        result: BuildResult,
        expected_parse_manifest: &FnParseManifest,
    ) {
        let build_manifest = self
            .build(mode.clone())
            .await
            .expect(format!("building {}", self.fixture_label()).as_str());
        self.expect_build_result(&mode, result, &expected_parse_manifest);
        self.verify_build_with_runtime(
            &self.build_output_dir(&mode),
            Some(mode),
            Some(build_manifest),
        )
        .await;
    }

    fn expect_build_result(
        &self,
        build_mode: &BuildMode,
        build_result: BuildResult,
        expected_parse_manifest: &FnParseManifest,
    ) {
        let build_dir = self.build_output_dir(build_mode);
        for expected_file in &build_result.files {
            let built_file_path: PathBuf = build_dir.join(&expected_file.path);
            let built_content = fs::read_to_string(&built_file_path).expect(
                format!(
                    "failed reading fixture {} build output file {}",
                    self.fixture_label(),
                    built_file_path.to_string_lossy(),
                )
                .as_str(),
            );
            let expected_content = match &expected_file.result {
                BuildFileOutput::Content(expected_content) => expected_content,
                BuildFileOutput::Identical => {
                    let original_file_path = self.fixture_dir.join(&expected_file.path);
                    &fs::read_to_string(&original_file_path).expect(
                        format!(
                            "failed reading fixture {} original file {}",
                            self.fixture_label(),
                            original_file_path.to_string_lossy()
                        )
                        .as_str(),
                    )
                }
            };
            assert_eq!(
                built_content.replace("\r\n", "\n").as_str(),
                expected_content.replace("\r\n", "\n").as_str(),
                "{} from fixture {} did not match expected content in build dir {}",
                expected_file.path.to_string_lossy(),
                self.fixture_label(),
                build_dir.to_string_lossy(),
            );
        }
        assert_eq!(
            build_result.files.len(),
            expected_parse_manifest.sources.len(),
            "fixture {} {} build has incorrect number of sources",
            self.fixture_label(),
            build_dir.file_name().unwrap().to_string_lossy(),
        );
    }

    async fn verify_parse(&self) -> Result<FnParseManifest, anyhow::Error> {
        let expected_parse_manifest = self.read_expected_parse_manifest();
        let parse_manifest = self
            .runtime
            .parse(FnParseSpec {
                entrypoint: self.spec.entrypoint.clone(),
                project_dir: self.fixture_dir.clone(),
                runtime: self.runtime.config(&self.fixture_dir),
            })
            .await?;
        for expected_source in &expected_parse_manifest.sources {
            match parse_manifest
                .sources
                .iter()
                .find(|source| source.path == expected_source.path)
            {
                None => panic!(
                    "parsing fixture {} did not contain source file {}",
                    self.fixture_label(),
                    expected_source.path.to_string_lossy(),
                ),
                Some(source) => {
                    assert_eq!(
                        source.imports,
                        expected_source.imports,
                        "fixture {} source file {} has incorrect imports",
                        self.fixture_label(),
                        source.path.to_string_lossy(),
                    );
                }
            }
        }
        assert_eq!(
            parse_manifest.sources.len(),
            expected_parse_manifest.sources.len(),
            "fixture {} parsing has incorrect number of sources",
            self.fixture_label(),
        );
        assert_eq!(
            parse_manifest.entrypoint,
            expected_parse_manifest.entrypoint,
            "fixture {} parsing has incorrect number of sources",
            self.fixture_label(),
        );
        Ok(expected_parse_manifest)
    }

    pub fn build_output_dir(&self, mode: &BuildMode) -> PathBuf {
        self.build_root_temp
            .child(match mode {
                BuildMode::Debug => "debug",
                BuildMode::Release => "release",
            })
            .join("build-fixture-test")
    }

    fn fixture_label(&self) -> String {
        self.fixture_dir
            .strip_prefix(env::current_dir().unwrap().join("fixtures"))
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    fn read_expected_parse_manifest(&self) -> FnParseManifest {
        let path = self.fixture_dir.join(".fixture").join("parse.json");
        debug_assert!(path.is_file());
        serde_json::from_str(fs::read_to_string(&path).unwrap().as_str())
            .map_err(|err| {
                anyhow!(
                    "failed parsing fixture {} parse.json: {err}",
                    self.fixture_label()
                )
            })
            .unwrap()
    }

    async fn verify_build_with_runtime(
        &self,
        project_dir: &Path,
        mode: Option<BuildMode>,
        build_manifest: Option<FnBuildManifest>,
    ) {
        let verify_result = match build_manifest {
            Some(build_manifest) => match build_manifest.output.paths.get(&self.spec.entrypoint) {
                Some(entrypoint) => self.runtime.verify(project_dir, entrypoint),
                None => self.runtime.verify(project_dir, &self.spec.entrypoint),
            },
            _ => self.runtime.verify(project_dir, &self.spec.entrypoint),
        };
        if let Some(result) = verify_result {
            let output = result.unwrap();
            if !output.status.success() {
                let verify_label = match mode {
                    Some(BuildMode::Debug) => "debug build",
                    Some(BuildMode::Release) => "release build",
                    None => "fixture directory",
                };
                let stderr = String::from_utf8(output.stderr).unwrap();
                let stdout = String::from_utf8(output.stdout).unwrap();
                panic!(
                    "fixture {} failed verifying {verify_label} with runtime with output:\n{}",
                    self.fixture_label(),
                    format!("~~STDERR~~\n{stderr}\n~~STDOUT~~\n{stdout}")
                );
            }
        }
    }
}
