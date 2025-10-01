use crate::BuildMode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Deserialized from .fixture/debug_build.json or .fixture/release_build.json.
#[derive(Deserialize, Serialize)]
pub struct BuildResult {
    pub files: Vec<BuildFile>,
}

/// A source file in a build output directory.
#[derive(Deserialize, Serialize)]
pub struct BuildFile {
    pub path: PathBuf,
    pub result: BuildFileOutput,
}

#[derive(Deserialize, Serialize)]
pub struct BuildFileOutput {
    pub content: BuildFileContent,
    pub path: Option<PathBuf>,
}

/// Expected content for a build output file.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildFileContent {
    Transformed(String),
    Identical,
}

impl BuildResult {
    pub fn read_json(fixture_dir: &Path, build_mode: &BuildMode) -> Option<BuildResult> {
        let filename = match build_mode {
            BuildMode::Debug => "build_debug.json",
            BuildMode::Release => "build_release.json",
        };
        let path = fixture_dir.join(".fixture").join(filename);
        if path.is_file() {
            Some(serde_json::from_str(fs::read_to_string(path).unwrap().as_str()).unwrap())
        } else {
            None
        }
    }
}
