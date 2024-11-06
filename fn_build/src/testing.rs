use crate::result::{FnBuild, FnBuildError, FnSources};
use crate::spec::{BuildMode, FnBuildSpec, FnParseSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::{env, fs};
use temp_dir::TempDir;

#[derive(Deserialize, Serialize)]
pub struct TestFixtureSpec {
    pub entrypoint: PathBuf,
}

#[derive(Deserialize, Serialize)]
pub struct BuildFile {
    pub path: PathBuf,
    pub result: BuildResult,
}

impl BuildFile {
    fn read_json(fixture_dir: &Path, build_mode: &BuildMode) -> Option<Vec<BuildFile>> {
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

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildResult {
    Content(String),
    Identical,
}

pub type BuildProcessResult<T> = Pin<Box<dyn Future<Output = Result<T, FnBuildError>> + Send>>;

pub trait BuildProcess {
    fn build(&self, build_spec: FnBuildSpec) -> BuildProcessResult<FnBuild>;

    fn parse(&self, parse_spec: FnParseSpec) -> BuildProcessResult<FnSources>;
}

fn collect_fixture_dirs(p: PathBuf) -> Vec<PathBuf> {
    debug_assert!(p.is_relative());
    let mut dirs = Vec::new();
    for dir_read_result in fs::read_dir(env::current_dir().unwrap().join(p)).unwrap() {
        let dir_read = dir_read_result.unwrap();
        let dir_read_path = dir_read.path();
        if dir_read_path.is_dir() {
            dirs.push(dir_read_path);
        }
    }
    dirs
}

pub async fn run_fixtures(build_process: Arc<Box<dyn BuildProcess>>, fixture_root_dir: PathBuf) {
    for dir in collect_fixture_dirs(fixture_root_dir) {
        let test_fixture = TestFixture::new(build_process.clone(), dir);
        if let Err(err) = test_fixture.test().await {
            panic!("{err}");
        }
    }
}

pub struct TestFixture {
    build_process: Arc<Box<dyn BuildProcess>>,
    fixture_dir: PathBuf,
    leak_build_dir: bool,
    spec: TestFixtureSpec,
}

impl TestFixture {
    pub fn new(build_process: Arc<Box<dyn BuildProcess>>, fixture_dir: PathBuf) -> Self {
        debug_assert!(fixture_dir.is_absolute());
        debug_assert!(fixture_dir.is_dir());
        let spec_json = fs::read_to_string(fixture_dir.join(".fixture").join("spec.json")).unwrap();
        let spec: TestFixtureSpec = serde_json::from_str(spec_json.as_str()).unwrap();
        Self {
            build_process,
            fixture_dir,
            leak_build_dir: false,
            spec,
        }
    }

    pub async fn test(&self) -> Result<(), anyhow::Error> {
        let temp_build_dir = TempDir::new()?;
        let build_dir_root = temp_build_dir.path().to_path_buf();
        if self.leak_build_dir {
            temp_build_dir.leak();
            println!(
                "{} {}",
                self.fixture_dir.to_string_lossy(),
                build_dir_root.to_string_lossy()
            );
        }
        self.expect_parse().await?;
        for (build_mode, build_files) in self.read_expected_build_results() {
            let build_dir = build_dir_root.join(match build_mode {
                BuildMode::Debug => "debug",
                BuildMode::Release => "release",
            });
            fs::create_dir(&build_dir)?;
            self.build(build_dir.clone(), build_mode.clone()).await?;
            self.expect_builds(build_dir.clone(), build_files)?;
        }
        Ok(())
    }

    async fn build(&self, build_dir: PathBuf, build_mode: BuildMode) -> Result<(), anyhow::Error> {
        self.build_process
            .build(FnBuildSpec {
                entrypoint: self.spec.entrypoint.clone(),
                mode: build_mode,
                output: build_dir,
                project_dir: self.fixture_dir.clone(),
            })
            .await?;
        Ok(())
    }

    fn expect_builds(
        &self,
        build_dir: PathBuf,
        build_files: Vec<BuildFile>,
    ) -> Result<(), anyhow::Error> {
        for build_file in build_files {
            let built_file_path = build_dir.join(&build_file.path);
            let built_content = fs::read_to_string(&built_file_path).expect(
                format!(
                    "failed reading build output file {}",
                    built_file_path.to_string_lossy()
                )
                .as_str(),
            );
            let expected_content = match build_file.result {
                BuildResult::Content(expected_content) => expected_content,
                BuildResult::Identical => {
                    let original_file_path = self.fixture_dir.join(&build_file.path);
                    fs::read_to_string(&original_file_path).expect(
                        format!(
                            "failed reading original file {}",
                            original_file_path.to_string_lossy()
                        )
                        .as_str(),
                    )
                }
            };
            assert_eq!(
                expected_content,
                built_content,
                "{} did not match expected content in build dir {}",
                build_file.path.to_string_lossy(),
                build_dir.to_string_lossy(),
            );
        }
        Ok(())
    }

    async fn expect_parse(&self) -> Result<(), anyhow::Error> {
        let expected_sources = self.read_expected_parse_result();
        let result_sources = self
            .build_process
            .parse(FnParseSpec {
                entrypoint: self.spec.entrypoint.clone(),
                project_dir: self.fixture_dir.clone(),
            })
            .await?;
        for expected_source in &expected_sources {
            match result_sources
                .iter()
                .find(|source| source.path == expected_source.path)
            {
                None => panic!(
                    "parsing fixture {} did not contain source file {}",
                    self.fixture_dir
                        .strip_prefix(env::current_dir()?.join("fixtures"))?
                        .to_string_lossy(),
                    expected_source.path.to_string_lossy(),
                ),
                Some(source) => {
                    assert_eq!(source.imports, expected_source.imports);
                }
            }
        }
        assert_eq!(expected_sources.len(), result_sources.len());
        Ok(())
    }

    fn read_expected_build_results(&self) -> HashMap<BuildMode, Vec<BuildFile>> {
        let mut result = HashMap::new();
        for build_mode in &[BuildMode::Debug, BuildMode::Release] {
            if let Some(build_files) = BuildFile::read_json(&self.fixture_dir, &build_mode) {
                result.insert(build_mode.clone(), build_files);
            }
        }
        debug_assert!(!result.is_empty());
        result
    }

    fn read_expected_parse_result(&self) -> FnSources {
        let path = self.fixture_dir.join(".fixture").join("parse.json");
        debug_assert!(path.is_file());
        serde_json::from_str(fs::read_to_string(path).unwrap().as_str()).unwrap()
    }
}
