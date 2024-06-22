use std::fs;
use std::path::PathBuf;

use rand::Rng;

use crate::aws::clients::AwsClients;
use crate::aws::config::load_sdk_config;
use crate::aws::operations::api_gateway::create_api;
use crate::aws::operations::iam::{create_lambda_role, get_account_id};
use crate::aws::state::DeployedProjectState;
use crate::aws::tasks::{DeployFnParams, SyncTask};
use crate::code::read::read_route_dir_for_lambdas;
use crate::{aws, code};

pub struct SyncOptions {
    pub api_id: Option<String>,
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
    println!("syncing project {}", sync_options.project_name);
    let sdk_config = load_sdk_config().await;
    let region = sdk_config.region().unwrap().to_owned();
    println!("aws sdk configured for region {region}");
    let sdk_clients = AwsClients::from(sdk_config);
    let account_id = get_account_id(&sdk_clients.iam).await?;
    let api_id = match sync_options.api_id {
        None => {
            let cached_api_id_path = PathBuf::from(".l3/api");
            if cached_api_id_path.exists() {
                fs::read_to_string(cached_api_id_path)?
            } else {
                println!("creating new api gateway");
                create_api(
                    &sdk_clients.api_gateway,
                    &sync_options.project_name,
                    &sync_options.stage_name,
                )
                .await?
            }
        }
        Some(api_id) => api_id,
    };
    println!("using api gateway {api_id}");
    fs::create_dir_all(format!(".l3/{api_id}"))?;
    fs::write(".l3/api", &api_id)?;

    let lambda_role = create_lambda_role(&sdk_clients.iam, &sync_options.project_name).await?;

    // todo deploy fn task build lambdas individually
    code::archive::create_archive()?;

    let local_fns =
        read_route_dir_for_lambdas(&sync_options.project_dir, &sync_options.project_name)?;
    let deployed_state = DeployedProjectState::fetch_state_from_aws(
        &sdk_clients,
        &sync_options.project_name,
        &api_id,
    )
    .await?;
    let mut sync_tasks: Vec<SyncTask> = Vec::new();

    let sync_id = create_sync_id();
    for lambda_fn in local_fns.values() {
        let deployed_components = deployed_state.get_deployed_components(&lambda_fn.route_key);
        sync_tasks.push(SyncTask::DeployFn(Box::new(DeployFnParams {
            account_id: account_id.clone(),
            api_id: api_id.clone(),
            deployed_components,
            lambda_fn: lambda_fn.clone(),
            lambda_role_arn: lambda_role.arn.clone(),
            region: region.to_string(),
            stage_name: sync_options.stage_name.clone(),
            sync_id: sync_id.clone(),
        })));
    }

    for task in sync_tasks {
        aws::tasks::exec(&sdk_clients, task).await?;
    }

    println!("\nLambdas deployed to API Gateway\n---");

    for (_, lambda_fn) in local_fns {
        println!(
            "{} https://{}.execute-api.{}.amazonaws.com/development/{}",
            lambda_fn.route_key.http_method, api_id, region, lambda_fn.route_key.http_path,
        );
    }

    Ok(())
}

fn create_sync_id() -> String {
    let mut rng = rand::thread_rng();
    (0..3)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}
