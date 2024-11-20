use l3_fn_build::runtime::node::NodeConfig;
use l3_fn_build::runtime::Runtime;
use l3_fn_build::{build_fn, BuildMode, FnBuildSpec, FnOutputConfig};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use temp_dir::TempDir;

#[tokio::main]
async fn main() {
    let project_dir = PathBuf::from(
        env::args()
            .nth(1)
            .unwrap_or_else(|| "fixtures/node/js/circular_imports".to_string()),
    );
    let out_dir = TempDir::new().unwrap();
    let node_config = NodeConfig::read_node_config(&project_dir).unwrap();
    let fn_build = build_fn(FnBuildSpec {
        entrypoint: PathBuf::from("routes/data/lambda.js"),
        handler_fn_name: "DELETE".to_string(),
        mode: BuildMode::Debug,
        output: FnOutputConfig {
            create_archive: false,
            build_root: out_dir.path().to_path_buf(),
        },
        project_dir: Arc::new(env::current_dir().unwrap().join(&project_dir)),
        runtime: Runtime::Node(Arc::new(node_config)),
    })
    .await
    .unwrap();
    println!(
        "l3_fn_build::build_fn output in temp dir from project dir {}:",
        project_dir.to_string_lossy()
    );
    for source in fn_build.sources {
        println!("   {}", source.path.to_string_lossy());
    }
}
