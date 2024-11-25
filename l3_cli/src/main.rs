use l3_fn_build::runtime::node::NodeConfig;
use l3_fn_build::runtime::Runtime;
use l3_fn_build::{FnEntrypoint, FnParseSpec};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    sync(Arc::new(
        env::current_dir()
            .unwrap()
            .join("fn_build/fixtures/node/js/http_routes/get_fn"),
    ))
    .await;
}

async fn sync(project_dir: Arc<PathBuf>) {
    assert!(project_dir.is_absolute());
    assert!(project_dir.is_dir());
    for entrypoint in collect_handlers(&project_dir).await {
        dbg!(entrypoint);
    }
}

async fn collect_handlers(project_dir: &Arc<PathBuf>) -> Vec<FnEntrypoint> {
    let mut join_set: JoinSet<Option<FnEntrypoint>> = JoinSet::new();
    for path in l3_api_base::collect_files(&project_dir.join("routes")) {
        if let Some(file_name) = path.file_name() {
            if matches!(
                file_name.to_string_lossy().as_ref(),
                "lambda.js" | "lambda.mjs" | "lambda.py" | "lambda.ts"
            ) {
                join_set.spawn(parse_entrypoint(
                    path.strip_prefix(project_dir.as_ref())
                        .unwrap()
                        .to_path_buf(),
                    project_dir.clone(),
                ));
            }
        }
    }
    let mut result = Vec::new();
    while let Some(join_result) = join_set.join_next().await {
        if let Some(handlers_found) = join_result.unwrap() {
            result.push(handlers_found);
        }
    }
    result
}

async fn parse_entrypoint(entrypoint: PathBuf, project_dir: Arc<PathBuf>) -> Option<FnEntrypoint> {
    let runtime = match entrypoint.extension().unwrap().to_string_lossy().as_ref() {
        "js" | "mjs" | "ts" => Runtime::Node(Arc::new(
            NodeConfig::read_node_config(project_dir.as_path()).unwrap(),
        )),
        "py" => Runtime::Python,
        _ => panic!(),
    };
    let entrypoint = l3_fn_build::parse_entrypoint(FnParseSpec {
        entrypoint: entrypoint.clone(),
        project_dir,
        runtime,
    })
    .await
    .unwrap();
    if entrypoint.handlers.is_empty() {
        None
    } else {
        Some(entrypoint)
    }
}
