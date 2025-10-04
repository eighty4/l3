use std::{fs, sync::Arc};

use crate::{parse_fn, testing::fixture::TestFixtureSpec, FnParseSpec};

pub async fn update_gold(spec: &Arc<TestFixtureSpec>) {
    let parse_result = parse_fn(FnParseSpec {
        entrypoint: spec.entrypoint.to_path_buf(),
        project_dir: spec.fixture_dir.clone(),
        runtime: spec.fixture_runtime(),
    })
    .await;
    let parse_error_path = spec.fixture_dir.join(".fixture/parse_error");
    let parse_json_path = spec.fixture_dir.join(".fixture/parse.json");
    _ = fs::remove_file(&parse_error_path);
    _ = fs::remove_file(&parse_json_path);
    let (p, c) = match parse_result {
        Ok(parse_manifest) => (
            parse_json_path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&parse_manifest).unwrap()
            ),
        ),
        Err(parse_error) => (parse_error_path, parse_error.to_string()),
    };
    fs::write(&p, c).unwrap();
}
