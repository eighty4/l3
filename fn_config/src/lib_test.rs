use std::{
    fs::{copy, read_dir, read_to_string},
    path::{Path, PathBuf},
    sync::Arc,
};

use temp_dir::TempDir;

use crate::LLLConfigs;

struct ConfigFixture {
    fixture_dir: PathBuf,
    temp_dir: TempDir,
}

impl ConfigFixture {
    fn for_dir(fixture_dir: PathBuf) -> Self {
        Self {
            fixture_dir,
            temp_dir: TempDir::new().unwrap(),
        }
    }

    fn copy_to_temp_dir(&self) {
        for p in read_dir(&self.fixture_dir).unwrap() {
            let from = p.unwrap().path();
            if !from.ends_with(".fixture") {
                let to = self
                    .temp_dir
                    .child(from.file_name().unwrap().to_string_lossy().to_string());
                copy(&from, &to).unwrap();
            }
        }
    }

    fn run_test(&self) {
        let mut configs = LLLConfigs::new(Arc::new(self.temp_dir.path().to_path_buf()));
        let update = configs.update_all_configs();
        assert!(update.config_errs.is_empty());
        let mut lambdas = configs.discrete_lambdas();
        let expected_lambdas: serde_json::Value = serde_json::from_str(
            &read_to_string(self.fixture_dir.join(".fixture/lambdas.json")).unwrap(),
        )
        .unwrap();
        for expected_lambdas_datum in expected_lambdas.as_array().unwrap() {
            let expected_lambda = expected_lambdas_datum.as_object().unwrap();
            let expected_lambda_name = expected_lambda.get("name").unwrap().as_str().unwrap();
            match lambdas.iter().position(|l| expected_lambda_name == l.name) {
                None => panic!("did not configure expected lambda: {expected_lambda_name}"),
                Some(i) => {
                    let lambda = lambdas.remove(i);
                    assert_eq!(
                        PathBuf::from(expected_lambda.get("source").unwrap().as_str().unwrap()),
                        lambda.source
                    );
                    assert_eq!(
                        expected_lambda.get("handler").unwrap().as_str().unwrap(),
                        lambda.handler
                    );
                }
            }
        }
        if !lambdas.is_empty() {
            panic!(
                "should not have configured lambdas: {}",
                lambdas
                    .iter()
                    .map(|l| l.name.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            )
        }
    }
}

#[test]
fn test_fn_config_fixtures() {
    for fixture_dir in collect_fixture_dirs(&PathBuf::from("fixtures")).unwrap() {
        let fixture = ConfigFixture::for_dir(fixture_dir);
        fixture.copy_to_temp_dir();
        fixture.run_test();
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
