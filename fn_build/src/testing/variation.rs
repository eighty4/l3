use std::{
    fmt,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

use l3_fn_config::NodeVersion;

#[derive(Clone)]
pub enum BuildVariation {
    Node(NodeBuildVariation),
}

#[derive(Clone)]
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

pub fn collect_variations(entrypoint: &Path) -> Option<Vec<BuildVariation>> {
    static NODE: OnceLock<Option<Vec<BuildVariation>>> = OnceLock::new();
    match entrypoint.extension().unwrap().to_str().unwrap() {
        "ts" | "js" | "mjs" => NODE.get_or_init(collect_node_variations).clone(),
        "py" => None,
        other => panic!("unsupported extension: {other}"),
    }
}

#[cfg(target_os = "windows")]
fn collect_node_variations() -> Option<Vec<BuildVariation>> {
    None
}

#[cfg(not(target_os = "windows"))]
fn collect_node_variations() -> Option<Vec<BuildVariation>> {
    use std::env;

    if env::var("SKIP_NVM").map(|v| v == "true").unwrap_or(false) {
        return None;
    }
    if Command::new("sh")
        .args(["-c", "stat ~/.nvm/nvm.sh"])
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
