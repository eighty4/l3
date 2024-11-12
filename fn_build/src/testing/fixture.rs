use crate::testing::result::{BuildFileOutput, BuildResult};
use crate::testing::runtimes::TestRuntime;
use crate::testing::spec::TestFixtureSpec;
use crate::{BuildMode, FnBuildSpec, FnManifest, FnParseSpec};
use anyhow::anyhow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{env, fs};
use temp_dir::TempDir;

pub struct TestFixture {
    build_dir_root: TempDir,
    runtime: Arc<Box<dyn TestRuntime>>,
    fixture_dir: Arc<PathBuf>,
    spec: TestFixtureSpec,
}

impl TestFixture {
    pub fn new(runtime: Arc<Box<dyn TestRuntime>>, fixture_dir: PathBuf) -> Self {
        debug_assert!(fixture_dir.is_absolute());
        debug_assert!(fixture_dir.is_dir());
        let spec = TestFixtureSpec::try_from(&fixture_dir).unwrap();
        Self {
            build_dir_root: TempDir::new().unwrap(),
            runtime,
            fixture_dir: Arc::new(fixture_dir),
            spec,
        }
    }

    pub async fn run(&self) {
        self.verify_build_with_runtime(&self.fixture_dir).await;
        let expected_manifest = self.verify_parse().await.unwrap();
        if let Some(debug_result) = BuildResult::read_json(&self.fixture_dir, &BuildMode::Debug) {
            self.run_build_test(BuildMode::Debug, debug_result, &expected_manifest)
                .await;
        }
        if let Some(release_result) = BuildResult::read_json(&self.fixture_dir, &BuildMode::Release)
        {
            self.run_build_test(BuildMode::Release, release_result, &expected_manifest)
                .await;
        }
    }

    async fn run_build_test(
        &self,
        mode: BuildMode,
        result: BuildResult,
        expected_manifest: &FnManifest,
    ) {
        let dir = self.build_dir_root.child(match mode {
            BuildMode::Debug => "debug",
            BuildMode::Release => "release",
        });
        fs::create_dir(&dir).unwrap();
        self.runtime
            .build(FnBuildSpec {
                function: FnParseSpec {
                    entrypoint: self.spec.entrypoint.clone(),
                    project_dir: self.fixture_dir.clone(),
                    runtime: self.runtime.config(&self.fixture_dir),
                },
                mode,
                output: dir.clone(),
            })
            .await
            .unwrap();
        self.expect_build_result(&dir, result, &expected_manifest);
        self.verify_build_with_runtime(&dir).await;
    }

    fn expect_build_result(
        &self,
        build_dir: &Path,
        build_result: BuildResult,
        expected_manifest: &FnManifest,
    ) {
        for expected_file in &build_result.files {
            let built_file_path = build_dir.join(&expected_file.path);
            let built_content = fs::read_to_string(&built_file_path).expect(
                format!(
                    "failed reading fixture {} build output file {}",
                    self.fixture_label(),
                    built_file_path.to_string_lossy()
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
                &built_content,
                expected_content,
                "{} from fixture {} did not match expected content in build dir {}",
                expected_file.path.to_string_lossy(),
                self.fixture_label(),
                build_dir.to_string_lossy(),
            );
        }
        assert_eq!(
            build_result.files.len(),
            expected_manifest.sources.len(),
            "fixture {} {} build has incorrect number of sources",
            self.fixture_label(),
            build_dir.file_name().unwrap().to_string_lossy(),
        );
    }

    async fn verify_parse(&self) -> Result<FnManifest, anyhow::Error> {
        let expected = self.read_expected_parse_manifest();
        let result = self
            .runtime
            .parse(FnParseSpec {
                entrypoint: self.spec.entrypoint.clone(),
                project_dir: self.fixture_dir.clone(),
                runtime: self.runtime.config(&self.fixture_dir),
            })
            .await?;
        for expected_source in &expected.sources {
            match result
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
            result.sources.len(),
            expected.sources.len(),
            "fixture {} parsing has incorrect number of sources",
            self.fixture_label(),
        );
        Ok(expected)
    }

    fn fixture_label(&self) -> String {
        self.fixture_dir
            .strip_prefix(env::current_dir().unwrap().join("fixtures"))
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    fn read_expected_parse_manifest(&self) -> FnManifest {
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

    async fn verify_build_with_runtime(&self, project_dir: &Path) {
        if let Some(result) = self.runtime.verify(project_dir, &self.spec.entrypoint) {
            let output = result.unwrap();
            if !output.status.success() {
                let verify_label = match project_dir.file_name().unwrap().to_string_lossy().as_ref()
                {
                    "debug" => "debug build",
                    "release" => "release build",
                    _ => "fixture directory",
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
