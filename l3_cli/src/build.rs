use crate::{LLLCommandRun, LLLCommandRunError, LLLCommandRunResult};
use clap::Parser;
use l3_fn_build::runtime::{node::NodeConfig, Runtime};
use l3_fn_build::{BuildMode, FnBuildManifest, FnBuildResult, FnBuildSpec, FnOutputConfig};
use l3_fn_config::{LLLConfigs, LambdaRuntimeSpec, LambdaSpec};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::{env, fs, process};
use tokio::task::JoinSet;

#[derive(Parser, Debug)]
pub struct BuildCommand {
    #[clap(
        long,
        default_value = "false",
        long_help = "Create a release build of Lambda functions"
    )]
    release: bool,
}

impl LLLCommandRun for BuildCommand {
    async fn run(&self) -> LLLCommandRunResult {
        let build_mode = if self.release {
            BuildMode::Release
        } else {
            BuildMode::Debug
        };
        let project_dir = Arc::new(env::current_dir().expect("cwd"));
        let build_root = project_dir.join(".l3");
        _ = fs::remove_dir_all(&build_root);
        let mut configs = LLLConfigs::new(project_dir.clone());
        let update_result = configs.update_all_configs();
        if !update_result.config_errs.is_empty() {
            for err in update_result.config_errs {
                println!("\x1b[31m✗\x1b[0m config error: {err}");
            }
            process::exit(1);
        }
        let lambda_specs = configs.discrete_lambdas();
        if lambda_specs.is_empty() {
            return Err(LLLCommandRunError::LambdasNotFound);
        }
        let node_config = Arc::new(NodeConfig::read_configs(&project_dir).unwrap());
        let mut join_set: JoinSet<(Arc<LambdaSpec>, FnBuildResult<FnBuildManifest>)> =
            JoinSet::new();
        for lambda_spec in lambda_specs {
            let runtime = match &lambda_spec.runtime {
                LambdaRuntimeSpec::Node => Runtime::Node(Some(node_config.clone())),
                LambdaRuntimeSpec::Python => Runtime::Python,
            };
            join_set.spawn(build_fn(
                lambda_spec.clone(),
                FnBuildSpec {
                    project_dir: project_dir.clone(),
                    runtime,
                    entrypoint: lambda_spec.source.clone(),
                    mode: build_mode.clone(),
                    handler_fn_name: lambda_spec.handler.clone(),
                    output: FnOutputConfig {
                        build_root: build_root.clone(),
                        create_archive: true,
                        dirname: lambda_spec.name.clone(),
                        use_build_mode: true,
                    },
                },
            ));
        }

        let mut errors: HashMap<String, String> = HashMap::new();
        let mut result: Vec<(Arc<LambdaSpec>, FnBuildResult<FnBuildManifest>)> = Vec::new();
        while let Some(join_result) = join_set.join_next().await {
            let (lambda_spec, build_result) = join_result.unwrap();
            if let Err(err) = &build_result {
                errors.insert(lambda_spec.name.clone(), err.to_string());
            }
            result.push((lambda_spec, build_result));
        }

        let build_count = result.len();
        write_build_manifest_json(&build_root, result);

        println!(
            "\x1b[32m✔\x1b[0m built {} lambdas successfully",
            build_count - errors.len()
        );
        for (lambda_name, err) in errors.iter() {
            println!("\x1b[31m✗\x1b[0m {lambda_name}: {err}");
        }
        Ok(())
    }
}

async fn build_fn(
    lambda_spec: Arc<LambdaSpec>,
    build_spec: FnBuildSpec,
) -> (Arc<LambdaSpec>, FnBuildResult<FnBuildManifest>) {
    (lambda_spec, l3_fn_build::build_fn(build_spec).await)
}

// todo `JSON.stringify(data, null, 4)` style output
fn write_build_manifest_json(
    build_root: &Path,
    lambda_builds: Vec<(Arc<LambdaSpec>, FnBuildResult<FnBuildManifest>)>,
) {
    let mut output: Vec<Value> = Vec::new();
    for (lambda_spec, build_result) in lambda_builds {
        output.push(build_lambda_value(lambda_spec, &build_result));
    }
    fs::write(
        build_root.join("l3_build.json"),
        serde_json::to_string(&output).unwrap(),
    )
    .unwrap();
}

// todo use project root relative paths for PathBufs in deserialized FnBuildManifest
fn build_lambda_value(
    lambda_spec: Arc<LambdaSpec>,
    build_result: &FnBuildResult<FnBuildManifest>,
) -> Value {
    let lambda_json = json!({
        "name": lambda_spec.name.clone(),
        "source": lambda_spec.source.clone(),
        "handler": lambda_spec.handler.clone(),
        // "runtime": match &lambda_spec.runtime {
        //     LambdaRuntimeSpec::Node(nc) => {
        //         json!({
        //             "name": "node",
        //             "version": match &nc.version {
        //                 Some(v) => Value::String(v.to_string()),
        //                 None => serde_json::Value::Null,
        //             }
        //         })
        //     }
        //     LambdaRuntimeSpec::Python(pc) => {
        //         json!({
        //             "name": "python",
        //             "version": match &pc.version {
        //                 Some(v) => Value::String(v.to_string()),
        //                 None => serde_json::Value::Null,
        //             }
        //         })
        //     }
        // },
        "build": json!({
            "manifest": build_result.as_ref().ok().map(|manifest| json!(manifest)).unwrap_or(Value::Null),
            "error": build_result.as_ref().err().map(|err| Value::String(err.to_string())).unwrap_or(Value::Null),
        }),
    });
    lambda_json
}
