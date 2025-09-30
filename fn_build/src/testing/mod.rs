mod fixture;
mod result;
mod runtimes;
mod spec;
mod utilities;

use crate::testing::fixture::TestFixture;
use std::env;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

pub use utilities::*;

#[allow(unused)]
enum FixtureTestMode {
    AllFixtures(PathBuf),
    SingleFixture(PathBuf),
}

#[tokio::test]
async fn test_single_fixture() {
    for fixture in collect_fixtures(FixtureTestMode::SingleFixture(PathBuf::from(
        "fixtures/node/ts/import_uses_js",
    )))
    .into_iter()
    .map(create_fixture)
    {
        fixture.run().await;
    }
}

#[tokio::test]
async fn test_node_fixtures() {
    for fixture in collect_fixtures(FixtureTestMode::AllFixtures(PathBuf::from("fixtures/node")))
        .into_iter()
        .map(create_fixture)
    {
        fixture.run().await;
    }
}

#[tokio::test]
async fn test_python_fixtures() {
    for fixture in collect_fixtures(FixtureTestMode::AllFixtures(PathBuf::from(
        "fixtures/python",
    )))
    .into_iter()
    .map(create_fixture)
    {
        fixture.run().await;
    }
}

#[tokio::test]
#[ignore]
async fn update_node_gold() {
    for fixture in collect_fixtures(FixtureTestMode::AllFixtures(PathBuf::from("fixtures/node")))
        .into_iter()
        .map(create_update_gold_fixture)
    {
        fixture.run().await;
    }
}

#[tokio::test]
#[ignore]
async fn update_python_gold() {
    for fixture in collect_fixtures(FixtureTestMode::AllFixtures(PathBuf::from(
        "fixtures/python",
    )))
    .into_iter()
    .map(create_update_gold_fixture)
    {
        fixture.run().await;
    }
}

fn collect_fixtures(mode: FixtureTestMode) -> Vec<PathBuf> {
    match mode {
        FixtureTestMode::AllFixtures(p) => {
            collect_fixture_dirs(&env::current_dir().unwrap().join(p)).unwrap()
        }
        FixtureTestMode::SingleFixture(p) => vec![env::current_dir().unwrap().join(p)],
    }
}

pub fn create_fixture(fixture_dir: PathBuf) -> TestFixture {
    TestFixture::new(env::current_dir().unwrap().join(fixture_dir))
}

fn create_update_gold_fixture(fixture_dir: PathBuf) -> TestFixture {
    TestFixture::gold_update(env::current_dir().unwrap().join(fixture_dir))
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
