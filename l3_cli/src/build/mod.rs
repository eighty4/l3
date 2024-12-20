use crate::collect::collect_handlers;
use crate::LLLCommandRunError::LambdasNotFound;
use crate::{LLLCommandRun, LLLCommandRunResult};
use clap::{Args, Parser};
use l3_fn_build::runtime::node::NodeConfig;
use l3_fn_build::runtime::Runtime;
use l3_fn_build::{BuildMode, FnBuildSpec, FnEntrypoint, FnHandler, FnOutputConfig, FnRouting};
use serde_json::{json, Value};
use std::sync::Arc;
use std::{env, fs};

#[derive(Parser, Debug)]
pub struct BuildCommand {
    #[clap(
        long,
        default_value = "false",
        long_help = "Create a release build of Lambda functions"
    )]
    release: bool,
    #[command(flatten)]
    target: BuildTarget,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct BuildTarget {
    #[clap(long)]
    all: bool,
    #[clap(long)]
    r#fn: Option<String>,
}

impl LLLCommandRun for BuildCommand {
    async fn run(&self) -> LLLCommandRunResult {
        let build_mode = if self.release {
            BuildMode::Release
        } else {
            BuildMode::Debug
        };

        if self.target.all {
            build_all_fns(build_mode).await
        } else {
            match &self.target.r#fn {
                None => panic!(),
                Some(r#fn) => build_fn(r#fn, build_mode).await,
            }
        }
    }
}

async fn build_fn(r#_fn: &str, _mode: BuildMode) -> LLLCommandRunResult {
    todo!();
}

async fn build_all_fns(mode: BuildMode) -> LLLCommandRunResult {
    let project_dir = Arc::new(env::current_dir().unwrap());
    let entrypoints = collect_handlers(&project_dir).await;
    if entrypoints.is_empty() {
        return Err(LambdasNotFound);
    }
    let node_config = Arc::new(NodeConfig::read_node_config(&project_dir).unwrap());
    let mut build_json: Vec<Value> = Vec::new();
    let build_root = project_dir.join(".l3/build").join(match mode {
        BuildMode::Debug => "debug",
        BuildMode::Release => "release",
    });
    fs::remove_dir_all(&build_root).expect("rm existing build dir");
    for entrypoint in entrypoints {
        for handler in &entrypoint.handlers {
            l3_fn_build::build_fn(FnBuildSpec {
                project_dir: project_dir.clone(),
                runtime: Runtime::Node(node_config.clone()),
                entrypoint: entrypoint.path.clone(),
                mode: mode.clone(),
                handler_fn_name: handler.fn_name.clone(),
                output: FnOutputConfig {
                    build_root: build_root.clone(),
                    create_archive: true,
                    use_build_mode: false,
                },
            })
            .await
            .unwrap();
            build_json.push(build_json_value(&entrypoint, handler));
        }
    }
    fs::write(
        build_root.join("l3_build.json"),
        serde_json::to_string(&build_json).unwrap(),
    )
    .unwrap();
    let rel_build_dir = build_root.strip_prefix(project_dir.as_ref()).unwrap();
    println!(
        "\x1b[0;32;1m✔\x1b[0m Lambda builds are in {}",
        rel_build_dir.to_string_lossy()
    );
    Ok(())
}

fn build_json_value(entrypoint: &FnEntrypoint, handler: &FnHandler) -> Value {
    let mut fn_json = json!({
        "fn_identifier": handler.to_fn_identifier(),
        "entrypoint_path": entrypoint.path.clone(),
        "handler_fn_name": handler.fn_name.clone(),
        "http_route": null,
    });
    if let FnRouting::HttpRoute(http_route) = &handler.routing {
        fn_json.as_object_mut().unwrap().insert(
            String::from("http_route"),
            json!({
                "http_method": http_route.method,
                "http_path": http_route.path,
            }),
        );
    }
    fn_json
}
