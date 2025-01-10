use l3_fn_build::runtime::node::NodeConfig;
use l3_fn_build::runtime::Runtime;
use l3_fn_build::{parse_fn, FnParseSpec};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let project_dir = PathBuf::from(
        env::args()
            .nth(1)
            .unwrap_or_else(|| "fixtures/node/js/circular_imports".to_string()),
    );
    let node_config = NodeConfig::read_node_config(&project_dir).unwrap();
    let fn_manifest = parse_fn(FnParseSpec {
        entrypoint: PathBuf::from("routes/data/lambda.js"),
        project_dir: Arc::new(env::current_dir().unwrap().join(&project_dir)),
        runtime: Runtime::Node(Some(Arc::new(node_config))),
    })
    .await
    .unwrap();
    println!(
        "l3_fn_build::parse_fn result for project dir {}:",
        project_dir.to_string_lossy()
    );
    for source in fn_manifest.sources {
        println!("   {}", source.path.to_string_lossy());
    }
}
