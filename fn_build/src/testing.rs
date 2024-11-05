use crate::result::{FnBuild, FnBuildError};
use crate::spec::{BuildMode, FnBuildSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::{env, fs};
use temp_dir::TempDir;

#[derive(Deserialize, Serialize)]
pub struct BuildFunction {
    pub path: PathBuf,
}

#[derive(Deserialize, Serialize)]
pub struct BuildFile {
    pub path: PathBuf,
    pub result: BuildResult,
}

impl BuildFile {
    fn read_json(fixture_dir: &Path, build_mode: &BuildMode) -> Option<Vec<BuildFile>> {
        let expect_filename = match build_mode {
            BuildMode::Debug => "expect_debug.json",
            BuildMode::Release => "expect_release.json",
        };
        let expect_path = fixture_dir.join(expect_filename);
        if expect_path.is_file() {
            Some(serde_json::from_str(fs::read_to_string(expect_path).unwrap().as_str()).unwrap())
        } else {
            None
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum BuildResult {
    Content(String),
    Identical,
}

pub type BuildProcessResult = Pin<Box<dyn Future<Output = Result<FnBuild, FnBuildError>> + Send>>;

pub trait BuildProcess {
    fn build(&self, build_spec: FnBuildSpec) -> BuildProcessResult;
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
}

impl TestFixture {
    pub fn new(build_process: Arc<Box<dyn BuildProcess>>, fixture_dir: PathBuf) -> Self {
        debug_assert!(fixture_dir.is_absolute());
        debug_assert!(fixture_dir.is_dir());
        Self {
            build_process,
            fixture_dir,
            leak_build_dir: false,
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
        for (build_mode, build_files) in self.read_expected_build_files() {
            let build_dir = build_dir_root.join(match build_mode {
                BuildMode::Debug => "debug",
                BuildMode::Release => "release",
            });
            fs::create_dir(&build_dir)?;
            self.build(build_dir.clone(), build_mode.clone()).await?;
            self.expect(build_dir.clone(), build_files)?;
        }
        Ok(())
    }

    async fn build(&self, build_dir: PathBuf, build_mode: BuildMode) -> Result<(), anyhow::Error> {
        let build_fn_json = fs::read_to_string(self.fixture_dir.join("build.json"))?;
        let build_fn: BuildFunction = serde_json::from_str(build_fn_json.as_str())?;
        self.build_process
            .build(FnBuildSpec {
                entrypoint: build_fn.path,
                mode: build_mode,
                output: build_dir,
                project_dir: self.fixture_dir.clone(),
            })
            .await?;
        Ok(())
    }

    fn expect(&self, build_dir: PathBuf, build_files: Vec<BuildFile>) -> Result<(), anyhow::Error> {
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

    fn read_expected_build_files(&self) -> HashMap<BuildMode, Vec<BuildFile>> {
        let mut result = HashMap::new();
        for build_mode in &[BuildMode::Debug, BuildMode::Release] {
            if let Some(build_files) = BuildFile::read_json(&self.fixture_dir, &build_mode) {
                result.insert(build_mode.clone(), build_files);
            }
        }
        debug_assert!(!result.is_empty());
        result
    }
}
