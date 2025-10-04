use crate::runtime::node::NodeConfig;
use crate::runtime::Runtime;
use crate::testing::update::update_gold;
use crate::testing::variation::{collect_variations, BuildVariation};
use crate::testing::verify_build::verify_build;
use crate::testing::verify_parse::verify_parse;
use crate::testing::verify_runtime::verify_with_runtime;
use crate::BuildMode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fmt, fs};

#[derive(Deserialize, Serialize)]
pub struct TestFixtureSpec {
    /// Fixture directory in //fn_build/fixtures.
    pub fixture_dir: Arc<PathBuf>,
    /// Defined in spec.json.
    pub entrypoint: PathBuf,
    /// Defined in spec.json.
    pub handler_fn_name: String,
}

impl TestFixtureSpec {
    pub fn from(fixture_dir: PathBuf) -> Arc<Self> {
        debug_assert!(fixture_dir.is_absolute());
        debug_assert!(fixture_dir.is_dir());
        let spec_json_path = fixture_dir.join(".fixture").join("spec.json");
        debug_assert!(spec_json_path.is_file());
        let spec_json = serde_json::from_str::<serde_json::Value>(
            fs::read_to_string(spec_json_path).unwrap().as_str(),
        )
        .unwrap();
        let entrypoint = PathBuf::from(
            spec_json
                .as_object()
                .unwrap()
                .get("entrypoint")
                .unwrap()
                .as_str()
                .unwrap(),
        );
        let handler_fn_name = spec_json
            .as_object()
            .unwrap()
            .get("handler_fn_name")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        Arc::new(Self {
            fixture_dir: Arc::new(fixture_dir),
            entrypoint,
            handler_fn_name,
        })
    }

    pub fn fixture_label(&self) -> String {
        self.fixture_dir
            .strip_prefix(env::current_dir().unwrap().join("fixtures"))
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    pub fn fixture_runtime(&self) -> Runtime {
        match self
            .entrypoint
            .extension()
            .unwrap()
            .to_string_lossy()
            .as_ref()
        {
            "js" | "mjs" | "ts" => Runtime::Node(Some(Arc::new(
                NodeConfig::read_configs(self.fixture_dir.as_path()).unwrap(),
            ))),
            "py" => Runtime::Python,
            _ => panic!(),
        }
    }
}

impl fmt::Display for TestFixtureSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fixture_label())
    }
}

enum FixtureTestScenario {
    Build {
        mode: BuildMode,
        variation: Option<BuildVariation>,
    },
    Parse,
    Runtime,
}

pub struct TestFixture {
    spec: Arc<TestFixtureSpec>,
}

impl TestFixture {
    pub fn new(fixture_dir: PathBuf) -> Self {
        let spec = TestFixtureSpec::from(fixture_dir);
        Self { spec }
    }

    pub async fn update_gold(self) {
        update_gold(&self.spec).await;
    }

    pub async fn run(&self) {
        let mut scenarios: Vec<FixtureTestScenario> = Vec::new();
        if !self.spec.fixture_dir.join("tsconfig.json").exists() {
            scenarios.push(FixtureTestScenario::Runtime)
        }
        scenarios.push(FixtureTestScenario::Parse);
        match collect_variations(&self.spec.entrypoint) {
            None => {
                scenarios.push(FixtureTestScenario::Build {
                    mode: BuildMode::Debug,
                    variation: None,
                });
                scenarios.push(FixtureTestScenario::Build {
                    mode: BuildMode::Release,
                    variation: None,
                });
            }
            Some(variations) => {
                for variation in variations {
                    scenarios.push(FixtureTestScenario::Build {
                        mode: BuildMode::Debug,
                        variation: Some(variation.clone()),
                    });
                    scenarios.push(FixtureTestScenario::Build {
                        mode: BuildMode::Release,
                        variation: Some(variation),
                    });
                }
            }
        }

        for scenario in scenarios {
            let spec = self.spec.clone();
            let result = match &scenario {
                FixtureTestScenario::Build { mode, variation } => {
                    verify_build(spec, mode, variation).await
                }
                FixtureTestScenario::Parse => verify_parse(spec).await,
                FixtureTestScenario::Runtime => verify_with_runtime(spec, None, None),
            };
            match result {
                Err(scenario_err) => {
                    dbg!(scenario_err);
                    panic!("scenario error");
                }
                Ok(_) => println!("scenario finished"),
            };
        }
    }
}
