mod fixture;
mod result;
mod runtimes;
mod spec;
mod utilities;

use crate::testing::fixture::TestFixture;
use crate::testing::runtimes::{TestNodeRuntime, TestRuntime};
use std::env;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use utilities::*;

#[allow(unused)]
enum FixtureTestMode {
    AllFixtures,
    SingleFixture(PathBuf),
}

#[tokio::test]
async fn test_fixtures() {
    let mode = FixtureTestMode::AllFixtures;
    // let mode = FixtureTestMode::SingleFixture(PathBuf::from("fixtures/node/js/http_route"));

    let runtime: Arc<Box<dyn TestRuntime>> = Arc::new(Box::new(TestNodeRuntime {}));
    match mode {
        FixtureTestMode::AllFixtures => {
            run_all_fixtures(PathBuf::from("fixtures/node"), runtime).await;
        }
        FixtureTestMode::SingleFixture(fixture) => {
            TestFixture::new(runtime, env::current_dir().unwrap().join(fixture))
                .run()
                .await;
        }
    }
}

fn create_fixture(fixture_dir: PathBuf, runtime: Arc<Box<dyn TestRuntime>>) -> TestFixture {
    TestFixture::new(runtime, env::current_dir().unwrap().join(fixture_dir))
}

pub fn create_node_fixture(fixture_dir: PathBuf) -> TestFixture {
    create_fixture(fixture_dir, Arc::new(Box::new(TestNodeRuntime {})))
}

pub async fn run_all_fixtures(fixture_root_dir: PathBuf, runtime: Arc<Box<dyn TestRuntime>>) {
    debug_assert!(fixture_root_dir.is_relative());
    debug_assert!(fixture_root_dir.starts_with("fixtures"));
    for dir in collect_fixture_dirs(&env::current_dir().unwrap().join(fixture_root_dir)).unwrap() {
        TestFixture::new(runtime.clone(), dir).run().await;
    }
}

fn collect_fixture_dirs(p: &Path) -> Result<Vec<PathBuf>, anyhow::Error> {
    let mut fixture_dirs = Vec::new();
    for dir_entry_result in read_dir(p)? {
        let dir_entry = dir_entry_result?;
        let p = dir_entry.path();
        if p.is_dir() {
            if is_fixture_dir(&p) {
                fixture_dirs.push(p);
            } else {
                fixture_dirs.append(&mut collect_fixture_dirs(&p)?);
            }
        }
    }
    Ok(fixture_dirs)
}

fn is_fixture_dir(p: &Path) -> bool {
    p.join(".fixture").is_dir() && !is_excluded_fixture_dir(p)
}

fn is_excluded_fixture_dir(p: &Path) -> bool {
    p.file_name().unwrap().to_string_lossy().starts_with('_')
}
