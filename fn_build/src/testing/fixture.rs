use crate::runtime::node::NodeConfig;
use crate::runtime::Runtime;
use crate::testing::result::{FixtureError, FixtureResult};
use crate::testing::scenarios::{collect_scenarios, FixtureTestScenario};
use crate::testing::update::update_gold;
use crate::testing::verify_build::verify_build;
use crate::testing::verify_parse::verify_parse;
use crate::testing::verify_runtime::verify_with_runtime;
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

    pub async fn run(&self) -> FixtureResult {
        let mut result = FixtureResult::from(self.spec.clone());
        for scenario in collect_scenarios(&self.spec) {
            let scenario_result = scenario.run(&self.spec).await;
            result.scenarios.push((scenario, scenario_result));
        }
        result
    }
}

impl FixtureTestScenario {
    async fn run(&self, spec: &Arc<TestFixtureSpec>) -> Option<FixtureError> {
        match self {
            FixtureTestScenario::Build { mode, variation } => {
                verify_build(spec, mode, variation).await
            }
            FixtureTestScenario::Parse => verify_parse(spec).await,
            FixtureTestScenario::Runtime => verify_with_runtime(spec, None, None),
        }
        .err()
    }
}
