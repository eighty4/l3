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

// run with `cargo test test_single_fixture -- -- --fixture fixtures/node/ts/import_uses_js`
#[tokio::test]
async fn test_single_fixture() {
    let mut fixture: Option<PathBuf> = None;
    let mut args = std::env::args().into_iter();
    while let Some(arg) = args.next() {
        if arg.as_str() == "--fixture" {
            match args.next().map(PathBuf::from) {
                Some(mut maybe_fixture) => {
                    if !maybe_fixture.is_absolute() && !maybe_fixture.starts_with("fixtures") {
                        maybe_fixture = PathBuf::from("fixtures").join(maybe_fixture);
                    }
                    if !maybe_fixture.is_dir() {
                        panic!(
                            "`--fixture {}` is not a directory",
                            maybe_fixture.to_string_lossy()
                        );
                    }
                    if !is_fixture_dir(&maybe_fixture) {
                        panic!(
                            "`--fixture {}` is not a fixture",
                            maybe_fixture.to_string_lossy()
                        );
                    }
                    fixture = Some(maybe_fixture);
                    break;
                }
                None => panic!("--fixture used without value"),
            }
        }
    }
    if let Some(fixture) = fixture.map(create_fixture) {
        fixture.run().await;
    }
}

#[tokio::test]
async fn test_node_fixtures() {
    for fixture in collect_fixtures("fixtures/node", false) {
        fixture.run().await;
    }
}

#[tokio::test]
async fn test_python_fixtures() {
    for fixture in collect_fixtures("fixtures/python", false) {
        fixture.run().await;
    }
}

#[tokio::test]
#[ignore]
async fn update_node_gold() {
    for fixture in collect_fixtures("fixtures/node", true) {
        fixture.run().await;
    }
}

#[tokio::test]
#[ignore]
async fn update_python_gold() {
    for fixture in collect_fixtures("fixtures/python", true) {
        fixture.run().await;
    }
}

fn collect_fixtures(p: &str, update_gold: bool) -> Vec<TestFixture> {
    collect_fixture_dirs(&env::current_dir().unwrap().join(p))
        .unwrap()
        .into_iter()
        .map(if update_gold {
            create_update_gold_fixture
        } else {
            create_fixture
        })
        .collect()
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
            if !is_excluded_fixture_dir(&p) {
                if is_fixture_dir(&p) {
                    fixture_dirs.push(p);
                } else {
                    fixture_dirs.append(&mut collect_fixture_dirs(&p)?);
                }
            }
        }
    }
    Ok(fixture_dirs)
}

fn is_fixture_dir(p: &Path) -> bool {
    p.join(".fixture").is_dir()
}

fn is_excluded_fixture_dir(p: &Path) -> bool {
    p.file_name().unwrap().to_string_lossy().starts_with('_')
}
