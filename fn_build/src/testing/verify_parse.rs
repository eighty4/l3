use anyhow::anyhow;
use std::{fs, sync::Arc};

use crate::testing::{fixture::TestFixtureSpec, result::FixtureError};
use crate::{parse_fn, FnParseManifest, FnParseResult, FnParseSpec};

pub async fn verify_parse(spec: &Arc<TestFixtureSpec>) -> Result<(), FixtureError> {
    let parse_result = parse_fn(FnParseSpec {
        entrypoint: spec.entrypoint.to_path_buf(),
        project_dir: spec.fixture_dir.clone(),
        runtime: spec.fixture_runtime(),
    })
    .await;
    match read_expected_parse_result(spec) {
        Ok(expected_manifest) => verify_successful_parse(parse_result, expected_manifest),
        Err(expected_error) => verify_parse_error(parse_result, expected_error),
    }
}

fn verify_successful_parse(
    parse_result: FnParseResult<FnParseManifest>,
    expected_manifest: FnParseManifest,
) -> Result<(), FixtureError> {
    let parse_manifest = parse_result.unwrap();
    for expected_source in &expected_manifest.sources {
        match parse_manifest
            .sources
            .iter()
            .find(|source| source.path == expected_source.path)
        {
            None => {
                return Err(FixtureError::ParseSuccess(format!(
                    "parse manifest did not contain source file {}",
                    expected_source.path.to_string_lossy(),
                )));
            }
            Some(source) => {
                if source.imports != expected_source.imports {
                    return Err(FixtureError::ParseSuccess(format!(
                        "parse manifest imports did not match for {}",
                        expected_source.path.to_string_lossy(),
                    )));
                }
            }
        }
    }
    if parse_manifest.sources.len() != expected_manifest.sources.len() {
        return Err(FixtureError::ParseSuccess(
            "parse manifest has too many sources".into(),
        ));
    }
    if parse_manifest.entrypoint != expected_manifest.entrypoint {
        return Err(FixtureError::ParseSuccess(
            "parse manifest entrypoint did not match".into(),
        ));
    }
    Ok(())
}

fn verify_parse_error(
    parse_result: FnParseResult<FnParseManifest>,
    expected: String,
) -> Result<(), FixtureError> {
    match parse_result {
        Ok(_) => Err(FixtureError::ParseError {
            expected,
            actual: None,
        }),
        Err(actual) => {
            if actual.to_string() != expected {
                Err(FixtureError::ParseError {
                    expected,
                    actual: Some(actual),
                })
            } else {
                Ok(())
            }
        }
    }
}

// Result represents the expected result of verify_parse and not the result of this function
fn read_expected_parse_result(spec: &Arc<TestFixtureSpec>) -> Result<FnParseManifest, String> {
    let parse_json_path = spec.fixture_dir.join(".fixture").join("parse.json");
    let parse_error_path = spec.fixture_dir.join(".fixture").join("parse_error");
    assert!(
        !parse_json_path.is_file() || !parse_error_path.is_file(),
        "cannot specify .fixture/parse.json and .fixture/parse_error for fixture {}",
        spec.fixture_dir.to_string_lossy()
    );
    assert!(
        parse_json_path.is_file() || parse_error_path.is_file(),
        "must specify .fixture/parse.json or .fixture/parse_error for fixture {}",
        spec.fixture_dir.to_string_lossy()
    );
    if parse_json_path.is_file() {
        Ok(
            serde_json::from_str(fs::read_to_string(&parse_json_path).unwrap().as_str())
                .map_err(|err| anyhow!("failed parsing fixture {spec} parse.json: {err}"))
                .unwrap(),
        )
    } else {
        Err(fs::read_to_string(&parse_error_path).unwrap())
    }
}
