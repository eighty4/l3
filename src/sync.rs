use crate::aws::preflight::AwsPreflightData;
use crate::aws::state::DeployedProjectState;
use crate::aws::tasks::SyncTask::RemoveFn;
use crate::aws::tasks::{exec_tasks, DeployFnParams, RemoveFnParams, SyncTask};
use crate::aws::{AwsApiConfig, AwsDataDir, AwsDeets};
use crate::code::build::BuildMode;
use crate::code::runtime::RuntimeConfig;
use crate::code::source::tree::SourceTree;
use crate::notification::LambdaNotification;
use crate::project::Lx3ProjectDeets;
use crate::ui::confirm::confirm;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;

pub struct SyncOptions {
    pub aws: AwsApiConfig,
    pub auto_confirm: bool,
    pub build_mode: BuildMode,
    pub clear_cache: bool,
    pub project_dir: PathBuf,
    pub project_name: String,
}

// todo validate aws credentials before starting workflows
//  verify iam policies permit sync workflows
// todo if api_id is none
//  check .l3 data dir for api state
//   prompt to confirm if a single api state
//   prompt multi select if multiple api states
//  otherwise query `aws lambda get-apis` and prompt multi select
pub(crate) async fn sync_project(sync_options: SyncOptions) -> Result<(), anyhow::Error> {
    let aws_preflight_data = AwsPreflightData::initialize(
        &sync_options.aws,
        &sync_options.project_dir,
        &sync_options.project_name,
    )
    .await?;
    let (runtime_config, runtime_config_api) = RuntimeConfig::new(sync_options.project_dir.clone());
    runtime_config_api.initialize_runtime_configs().await;
    let project_deets = Arc::new(
        Lx3ProjectDeets::builder()
            .aws_deets(AwsDeets::from(aws_preflight_data))
            .build_mode(sync_options.build_mode.clone())
            .runtime_config(runtime_config)
            .build(
                sync_options.project_dir.clone(),
                sync_options.project_name.clone(),
            ),
    );

    println!("λλλ sync");
    println!("  project: {}", &project_deets.project_name);
    println!("  region: {}", &project_deets.aws.region);
    println!("  api id: {}", &project_deets.aws.api.id);

    if !sync_options.auto_confirm && !confirm("\n  Continue with syncing?")? {
        println!("  Cancelling sync operations!");
        process::exit(0);
    }

    if sync_options.clear_cache {
        println!(
            "\nClearing cache at .l3/aws/{} and re-syncing",
            &project_deets.aws.api.id
        );
        AwsDataDir::clear_cache(&project_deets.aws.api.id, &project_deets.project_dir);
    }

    AwsDataDir::cache_api_id(&project_deets.project_dir, &project_deets.aws.api.id)?;

    let (notification_tx, _notification_rx) = unbounded_channel::<LambdaNotification>();
    let (source_tree, sources_api) = SourceTree::new(notification_tx, project_deets.clone());
    sources_api.refresh_routes().await?;

    let mut deployed_state = DeployedProjectState::fetch_from_aws(
        &project_deets.aws.sdk_clients,
        &project_deets.project_name,
        &project_deets.aws.api.id,
    )
    .await?;
    let mut sync_tasks: Vec<SyncTask> = Vec::new();
    let lambda_fns = { source_tree.lock().unwrap().lambda_fns() };

    println!("\nSyncing {} lambdas", lambda_fns.len());
    for lambda_fn in &lambda_fns {
        sync_tasks.push(SyncTask::DeployFn(Box::new(DeployFnParams {
            components: deployed_state
                .rm_deployed_components(&lambda_fn.route_key, &lambda_fn.fn_name),
            lambda_fn: lambda_fn.clone(),
            project_deets: project_deets.clone(),
            publish_fn_updates: false,
        })));
    }

    let removing = deployed_state.collect_deployed_components(&project_deets.project_name);
    if !removing.is_empty() {
        println!("Removing {} lambdas", removing.len());
        for components in removing {
            sync_tasks.push(RemoveFn(Box::new(RemoveFnParams {
                api_id: project_deets.aws.api.id.clone(),
                components,
            })))
        }
    }

    exec_tasks(&project_deets.aws.sdk_clients, sync_tasks).await?;

    println!("\nLambdas deployed to API Gateway\n---");

    for lambda_fn in lambda_fns {
        println!(
            "{} https://{}.execute-api.{}.amazonaws.com/development/{}",
            lambda_fn.route_key.http_method,
            &project_deets.aws.api.id,
            &project_deets.aws.region,
            lambda_fn.route_key.http_path,
        );
    }

    Ok(())
}
