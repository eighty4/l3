use crate::testing::fixture::TestFixtureSpec;
use crate::testing::scenarios::FixtureTestScenario;
use crate::{FnBuildError, FnParseError};
use std::fmt;
use std::path::PathBuf;
use std::process::Output;
use std::sync::Arc;

pub struct FixtureResult {
    pub spec: Arc<TestFixtureSpec>,
    pub scenarios: Vec<(FixtureTestScenario, Option<FixtureError>)>,
}

#[derive(Debug)]
pub enum FixtureError {
    Build(FnBuildError),
    ParseSuccess(String),
    ParseError {
        expected: String,
        actual: Option<FnParseError>,
    },
    Runtime {
        bin: PathBuf,
        output: Output,
    },
}

impl FixtureResult {
    pub fn has_error(&self) -> bool {
        self.scenarios.iter().any(|(_, err)| err.is_some())
    }

    pub fn print(&self) {
        println!("{}", self.spec);
        for (scenario, result) in &self.scenarios {
            println!(
                "{}",
                match result {
                    None => format!("\x1b[32m✔\x1b[0m {scenario}"),
                    Some(fixture_error) => format!(
                        "\x1b[31m✗\x1b[0m {scenario}: {}",
                        match fixture_error {
                            FixtureError::Build(build_error) => build_error.to_string(),
                            FixtureError::ParseSuccess(msg) => msg.clone(),
                            FixtureError::ParseError { expected, actual } => match actual {
                                None => format!("parse did not error!\n   expected: {}", expected),
                                Some(actual) => format!(
                                    "parse error did not match expected!\n   {}\n   {}",
                                    expected, actual
                                ),
                            },
                            FixtureError::Runtime { bin, output } => format!(
                                "{}\n~~~stdout~~~\n{}\n~~~stderr~~~\n{}\n~~~",
                                bin.to_string_lossy(),
                                str::from_utf8(&output.stdout).unwrap(),
                                str::from_utf8(&output.stderr).unwrap()
                            ),
                        }
                    ),
                }
            );
        }
    }
}

impl From<Arc<TestFixtureSpec>> for FixtureResult {
    fn from(spec: Arc<TestFixtureSpec>) -> Self {
        Self {
            spec,
            scenarios: Vec::new(),
        }
    }
}

impl fmt::Display for FixtureTestScenario {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FixtureTestScenario::Runtime => "runtime".into(),
                FixtureTestScenario::Parse => "parse_fn".into(),
                FixtureTestScenario::Build { mode, variation } => format!(
                    "build_fn ({mode}, {})",
                    variation
                        .as_ref()
                        .map(|v| format!("{v}"))
                        .unwrap_or("default".into())
                ),
            }
        )
    }
}
