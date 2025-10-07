use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use l3_fn_config::Language;

use crate::testing::{fixture::TestFixtureSpec, result::FixtureError};
use crate::FnBuildManifest;

#[cfg(target_os = "windows")]
mod bin {
    pub const NODE: &str = "node.exe";
    pub const PYTHON: &str = "python.exe";
}

#[cfg(not(target_os = "windows"))]
mod bin {
    pub const NODE: &str = "node";
    pub const PYTHON: &str = "python3";
}

pub fn verify_with_runtime(
    spec: &Arc<TestFixtureSpec>,
    build: Option<(PathBuf, FnBuildManifest)>,
    bin_override: Option<PathBuf>,
) -> Result<(), FixtureError> {
    match build {
        None => run_runtime(
            spec.fixture_dir.as_ref(),
            &spec.entrypoint,
            &spec.language,
            bin_override,
        ),
        Some((build_dir, build_manifest)) => {
            let entrypoint = build_manifest.output.paths.get(&spec.entrypoint).unwrap();
            run_runtime(&build_dir, entrypoint, &spec.language, bin_override)
        }
    }
}

fn run_runtime(
    project_dir: &Path,
    entrypoint: &Path,
    language: &Language,
    bin_override: Option<PathBuf>,
) -> Result<(), FixtureError> {
    assert!(
        bin_override
            .as_ref()
            .map(|bp| bp.is_absolute())
            .unwrap_or(true),
        "bin_override {} was not absolute",
        bin_override.unwrap().to_string_lossy()
    );
    let bin = match language {
        Language::JavaScript | Language::TypeScript => {
            bin_override.unwrap_or_else(|| PathBuf::from(bin::NODE))
        }
        Language::Python => bin_override.unwrap_or_else(|| PathBuf::from(bin::PYTHON)),
    };
    let output = match Command::new(&bin)
        .arg(entrypoint)
        .current_dir(project_dir)
        .output()
    {
        Err(err) => panic!("error running {}: {err}", bin.to_string_lossy(),),
        Ok(output) => output,
    };
    if output.status.success() {
        Ok(())
    } else {
        Err(FixtureError::Runtime { bin, output })
    }
}
