use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, process};

use anyhow::anyhow;

use crate::aws::clients::AwsClients;
use crate::aws::config::load_sdk_config;
use crate::aws::operations::api_gateway::{create_api, does_api_exist};
use crate::aws::operations::iam::{create_lambda_role, get_account_id};
use crate::aws::state::DeployedProjectState;
use crate::aws::tasks::SyncTask::RemoveFn;
use crate::aws::tasks::{DeployFnParams, RemoveFnParams, SyncTask};
use crate::code::env::EnvVarSources;
use crate::code::parse::parse_source_file;
use crate::code::read::recursively_read_dirs;
use crate::code::source::SourceFile;
use crate::config::{read_api_id_from_data_dir, write_api_id_to_data_dir};
use crate::lambda::{HttpMethod, LambdaFn, RouteKey};
use crate::{aws, ui};

pub struct SyncOptions {
    pub api_id: Option<String>,
    pub auto_confirm: bool,
    pub clear_cache: bool,
    pub project_dir: PathBuf,
    pub project_name: String,
    pub stage_name: String,
}

// todo validate aws credentials before starting workflows
//  verify iam policies permit sync workflows
// todo if api_id is none
//  check .l3 data dir for api state
//   prompt to confirm if a single api state
//   prompt multi select if multiple api states
//  otherwise query `aws lambda get-apis` and prompt multi select
pub(crate) async fn sync_project(sync_options: SyncOptions) -> Result<(), anyhow::Error> {
    let sdk_config = load_sdk_config().await;
    let region = sdk_config.region().unwrap().to_owned();
    let sdk_clients = AwsClients::from(sdk_config);
    let api_id = validate_or_create_api(&sdk_clients, &sync_options).await?;

    println!("λλλ sync");
    println!("  project: {}", sync_options.project_name);
    println!("  region: {region}");
    println!("  api id: {api_id}");
    if !sync_options.auto_confirm && !ui::confirm("  Continue with syncing?")? {
        println!("  Cancelling sync operations!");
        process::exit(0);
    }

    if sync_options.clear_cache {
        println!("\nClearing cache at .l3/{api_id} and re-syncing");
        let _ = fs::remove_dir_all(PathBuf::from(".l3").join(&api_id));
    }

    write_api_id_to_data_dir(&api_id)?;
    let account_id = get_account_id(&sdk_clients.iam).await?;
    let lambda_role = create_lambda_role(&sdk_clients.iam, &sync_options.project_name).await?;

    let lambdas =
        read_route_dir_for_lambdas(&sync_options.project_dir, &sync_options.project_name)?;
    let mut deployed_state =
        DeployedProjectState::fetch_from_aws(&sdk_clients, &sync_options.project_name, &api_id)
            .await?;
    let mut sync_tasks: Vec<SyncTask> = Vec::new();

    println!("\nSyncing {} lambdas", lambdas.len());
    for lambda_fn in &lambdas {
        sync_tasks.push(SyncTask::DeployFn(Box::new(DeployFnParams {
            account_id: account_id.clone(),
            api_id: api_id.clone(),
            components: deployed_state
                .rm_deployed_components(&lambda_fn.route_key, &lambda_fn.fn_name),
            lambda_fn: lambda_fn.clone(),
            lambda_role_arn: lambda_role.arn.clone(),
            project_dir: sync_options.project_dir.clone(),
            publish_fn_updates: false,
            region: region.to_string(),
            stage_name: sync_options.stage_name.clone(),
        })));
    }

    let removing = deployed_state.collect_deployed_components(&sync_options.project_name);
    if !removing.is_empty() {
        println!("Removing {} lambdas", removing.len());
        for components in removing {
            sync_tasks.push(RemoveFn(Box::new(RemoveFnParams {
                api_id: api_id.clone(),
                components,
            })))
        }
    }

    aws::tasks::exec(&sdk_clients, sync_tasks).await?;

    println!("\nLambdas deployed to API Gateway\n---");

    for lambda_fn in lambdas {
        println!(
            "{} https://{}.execute-api.{}.amazonaws.com/development/{}",
            lambda_fn.route_key.http_method, api_id, region, lambda_fn.route_key.http_path,
        );
    }

    Ok(())
}

async fn validate_or_create_api(
    sdk_clients: &AwsClients,
    sync_options: &SyncOptions,
) -> Result<String, anyhow::Error> {
    let maybe_api_id = if sync_options.api_id.is_none() {
        read_api_id_from_data_dir()?
    } else {
        sync_options.api_id.clone()
    };
    match maybe_api_id {
        None => {
            println!("creating new api gateway");
            create_api(
                &sdk_clients.api_gateway,
                &sync_options.project_name,
                &sync_options.stage_name,
            )
            .await
        }
        Some(api_id) => match does_api_exist(&sdk_clients.api_gateway, &api_id).await {
            Ok(api_exists) => {
                if api_exists {
                    Ok(api_id)
                } else {
                    Err(anyhow!("api {api_id} does not exist"))
                }
            }
            Err(err) => Err(anyhow!("error verifying api {api_id} exists: {err}")),
        },
    }
}

// todo testing
fn read_route_dir_for_lambdas(
    project_dir: &Path,
    project_name: &String,
) -> Result<Vec<LambdaFn>, anyhow::Error> {
    let mut lambdas = Vec::new();
    for path in recursively_read_dirs(&PathBuf::from("routes"))? {
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        match file_name.as_ref() {
            "lambda.ts" | "lambda.js" | "lambda.mjs" => {}
            _ => {
                // todo warning about unresolvable env and non route source files
                continue;
            }
        }
        let source_file = parse_source_file(&path, project_dir)?;
        let handler_fns = collect_handler_fn_names(&source_file)?;
        if handler_fns.is_empty() {
            // todo warning if env files without any lambdas in a route dir
            continue;
        }
        let http_path = RouteKey::extract_http_path(&path).unwrap();
        // todo move to SourceFile
        for (http_method, handler_fn) in handler_fns {
            let route_key = RouteKey::new(http_method, http_path.clone());
            let env_var_sources = EnvVarSources::new(project_dir, &route_key)?;
            lambdas.push(LambdaFn::new(
                env_var_sources,
                handler_fn,
                path.clone(),
                project_name,
                route_key,
            ));
        }
    }
    Ok(lambdas)
}

// todo move to SourceFile
fn collect_handler_fn_names(
    source_file: &SourceFile,
) -> Result<HashMap<HttpMethod, String>, anyhow::Error> {
    let mut handler_fns = HashMap::new();
    for exported_fn in &source_file.exported_fns {
        if let Ok(http_method) = HttpMethod::try_from(exported_fn.as_str()) {
            if handler_fns.contains_key(&http_method) {
                return Err(anyhow!(
                    "multiple {http_method} functions found in source file {}",
                    source_file.path.file_name().unwrap().to_string_lossy()
                ));
            }
            handler_fns.insert(http_method, exported_fn.clone());
        }
    }
    Ok(handler_fns)
}
