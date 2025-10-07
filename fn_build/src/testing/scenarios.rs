use std::{
    fmt,
    path::PathBuf,
    process::Command,
    sync::{Arc, OnceLock},
};

use l3_fn_config::{Language, NodeVersion};

use crate::{testing::fixture::TestFixtureSpec, BuildMode};

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum FixtureTestScenario {
    Build {
        mode: BuildMode,
        variation: Option<BuildVariation>,
    },
    Parse,
    Runtime,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum BuildVariation {
    Node(NodeBuildVariation),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NodeBuildVariation {
    pub bin: PathBuf,
    pub version: NodeVersion,
}

impl fmt::Display for BuildVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildVariation::Node(node_variation) => {
                write!(f, "node-{}", node_variation.version)
            }
        }
    }
}

pub fn collect_scenarios(spec: &Arc<TestFixtureSpec>) -> Vec<FixtureTestScenario> {
    let skip_runtime = spec
        .fixture_dir
        .join(".fixture")
        .join("runtime.skip")
        .exists();
    let skip_debug = spec
        .fixture_dir
        .join(".fixture")
        .join("build_debug.skip")
        .exists();
    let skip_release = spec
        .fixture_dir
        .join(".fixture")
        .join("build_release.skip")
        .exists();
    let mut scenarios: Vec<FixtureTestScenario> = Vec::new();
    if !skip_runtime && !spec.fixture_dir.join("tsconfig.json").exists() {
        scenarios.push(FixtureTestScenario::Runtime);
    }
    scenarios.push(FixtureTestScenario::Parse);
    if skip_debug && skip_release {
        return scenarios;
    }
    match collect_variations(spec) {
        None => {
            if !skip_debug {
                scenarios.push(FixtureTestScenario::Build {
                    mode: BuildMode::Debug,
                    variation: None,
                });
            }
            if !skip_release {
                scenarios.push(FixtureTestScenario::Build {
                    mode: BuildMode::Release,
                    variation: None,
                });
            }
        }
        Some(variations) => {
            for variation in variations {
                if !skip_debug {
                    scenarios.push(FixtureTestScenario::Build {
                        mode: BuildMode::Debug,
                        variation: Some(variation.clone()),
                    });
                }
                if !skip_release {
                    scenarios.push(FixtureTestScenario::Build {
                        mode: BuildMode::Release,
                        variation: Some(variation),
                    });
                }
            }
        }
    }
    dbg!(&scenarios);
    scenarios
}

fn collect_variations(spec: &Arc<TestFixtureSpec>) -> Option<Vec<BuildVariation>> {
    static NODE: OnceLock<Option<Vec<BuildVariation>>> = OnceLock::new();
    match spec.language {
        Language::JavaScript | Language::TypeScript => {
            NODE.get_or_init(collect_node_variations).clone()
        }
        Language::Python => None,
    }
}

#[cfg(target_os = "windows")]
fn collect_node_variations() -> Option<Vec<BuildVariation>> {
    None
}

#[cfg(not(target_os = "windows"))]
fn collect_node_variations() -> Option<Vec<BuildVariation>> {
    use std::{env, process::Stdio};

    if env::var("SKIP_NVM").map(|v| v == "true").unwrap_or(false) {
        return None;
    }
    if Command::new("sh")
        .args(["-c", "stat ~/.nvm/nvm.sh"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_err()
    {
        panic!("nvm is required for node fixtures");
    }
    Some(Vec::from(
        [
            NodeVersion::Eighteen,
            NodeVersion::Twenty,
            NodeVersion::TwentyTwo,
            NodeVersion::TwentyFour,
        ]
        .map(|version| {
            let output = Command::new("sh")
                .args([
                    "-c",
                    format!("\\. ~/.nvm/nvm.sh && nvm which {version}").as_str(),
                ])
                .output()
                .unwrap();
            let stdout = String::from_utf8(output.stdout)
                .map(|s| s.trim().to_string())
                .unwrap();
            if stdout.is_empty() {
                panic!("nvm is not writing to stdio");
            }
            BuildVariation::Node(NodeBuildVariation {
                bin: PathBuf::from(stdout),
                version,
            })
        }),
    ))
}
