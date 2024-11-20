use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct TestFixtureSpec {
    pub entrypoint: PathBuf,
    pub handler_fn_name: String,
}

impl TryFrom<&PathBuf> for TestFixtureSpec {
    type Error = anyhow::Error;

    fn try_from(fixture_dir: &PathBuf) -> Result<Self, Self::Error> {
        let spec_json_path = fixture_dir.join(".fixture").join("spec.json");
        let spec_json = fs::read_to_string(spec_json_path)?;
        Ok(serde_json::from_str::<Self>(spec_json.as_str())?)
    }
}
