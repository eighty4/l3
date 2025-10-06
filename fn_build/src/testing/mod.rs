mod fixture;
mod result;
mod scenarios;
mod update;
mod utilities;
mod verify_build;
mod verify_parse;
mod verify_runtime;

use crate::testing::fixture::TestFixture;
use std::env;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

pub use utilities::*;

// run with `cargo test test_single_fixture -- -- --fixture fixtures/node/ts/import_uses_js`
#[tokio::test]
async fn test_single_fixture() {
    if skip_fixtures() || is_update_gold() {
        return;
    }
    let mut fixture: Option<PathBuf> = None;
    let mut args = env::args();
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
        fixture.run().await.print();
    }
}

#[tokio::test]
async fn test_errors_fixtures() {
    if skip_fixtures() || is_update_gold() {
        return;
    }
    run_fixtures(collect_fixtures("fixtures/errors")).await;
}

#[tokio::test]
async fn test_node_fixtures() {
    if skip_fixtures() || is_update_gold() {
        return;
    }
    run_fixtures(collect_fixtures("fixtures/node")).await;
}

#[tokio::test]
async fn test_python_fixtures() {
    if skip_fixtures() || is_update_gold() {
        return;
    }
    run_fixtures(collect_fixtures("fixtures/python")).await;
}

async fn run_fixtures(fixtures: Vec<TestFixture>) {
    let mut has_errors = false;
    for fixture in fixtures {
        let result = fixture.run().await;
        if result.has_error() {
            has_errors = true;
            result.print();
        }
    }
    assert!(!has_errors);
}

#[tokio::test]
async fn update_node_gold() {
    if !is_update_gold() {
        return;
    }
    for fixture in collect_fixtures("fixtures/node") {
        fixture.update_gold().await;
    }
}

#[tokio::test]
async fn update_python_gold() {
    if !is_update_gold() {
        return;
    }
    for fixture in collect_fixtures("fixtures/python") {
        fixture.update_gold().await;
    }
}

fn skip_fixtures() -> bool {
    env::var("SKIP_BUILD_FIXTURES")
        .map(|v| v == "true")
        .unwrap_or(false)
}

fn is_update_gold() -> bool {
    for arg in std::env::args() {
        if arg.as_str() == "--update-gold" {
            return true;
        }
    }
    false
}

fn collect_fixtures(p: &str) -> Vec<TestFixture> {
    collect_fixture_dirs(&env::current_dir().unwrap().join(p))
        .unwrap()
        .into_iter()
        .map(create_fixture)
        .collect()
}

pub fn create_fixture(fixture_dir: PathBuf) -> TestFixture {
    TestFixture::new(env::current_dir().unwrap().join(fixture_dir))
}

fn collect_fixture_dirs(p: &Path) -> Result<Vec<PathBuf>, anyhow::Error> {
    let mut fixture_dirs = Vec::new();
    for dir_entry_result in read_dir(p)? {
        let dir_entry = dir_entry_result?;
        let p = dir_entry.path();
        if p.is_dir() && !is_excluded_fixture_dir(&p) {
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
    p.join(".fixture").is_dir()
}

fn is_excluded_fixture_dir(p: &Path) -> bool {
    p.file_name().unwrap().to_string_lossy().starts_with('_')
}
