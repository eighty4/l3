use crate::testing::variation::BuildVariation;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Output;
use std::sync::Arc;

pub struct FixtureResult {
    fixture_dir: Arc<PathBuf>,
    runtime: Option<Result<(), anyhow::Error>>,
    build_variation: HashMap<Option<BuildVariation>, Result<(), anyhow::Error>>,
}

#[derive(Debug)]
pub enum FixtureError {
    Runtime { bin: PathBuf, output: Output },
}
