use crate::aws::preflight::AwsPreflightData;
use crate::aws::tasks::AwsTaskTranslation;
use crate::aws::{AwsApiGatewayConfig, AwsDataDir, AwsProject};
use crate::code::build::BuildMode;
use crate::code::runtime::RuntimeConfig;
use crate::code::source::tree::SourceTree;
use crate::project::Lx3Project;
use crate::task::executor::TaskExecutor;
use crate::task::pool::TaskPool;
use crate::task::LambdaTaskKind;
use crate::ui::dev::print_notification;
use crate::ui::prompt::confirm::prompt_for_confirmation;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;

pub struct SyncOptions {
    pub aws: AwsApiGatewayConfig,
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
    let (project, mut notification_rx) = Lx3Project::builder()
        .aws(Arc::new(AwsProject::from(aws_preflight_data)))
        .build_mode(sync_options.build_mode.clone())
        .runtime_config(runtime_config)
        .build(
            sync_options.project_dir.clone(),
            sync_options.project_name.clone(),
        );
    let aws = project.aws();

    println!("λλλ sync");
    println!("  project: {}", &project.name);
    println!("  region: {}", aws.sdk_clients.region());
    println!("  api id: {}", &aws.api.id);

    if !sync_options.auto_confirm && !prompt_for_confirmation("\n  Continue with syncing?") {
        println!("  Cancelling sync operations!");
        process::exit(0);
    }

    if sync_options.clear_cache {
        println!("\nClearing cache at .l3/aws/{} and re-syncing", &aws.api.id);
        AwsDataDir::clear_cache(&aws.api.id, &project.dir);
    }

    AwsDataDir::cache_api_id(&project.dir, &aws.api.id)?;

    spawn(async move {
        while let Some(notification) = notification_rx.recv().await {
            print_notification(&notification);
        }
    });

    let (source_tree, sources_api) = SourceTree::new(project.clone());
    sources_api.refresh_routes().await?;
    aws.resources.refresh_state().await?;

    let lambda_fns = { source_tree.lock().unwrap().lambda_fns() };

    let task_pool = TaskPool::new(TaskExecutor::new(
        project.clone(),
        source_tree.clone(),
        Box::new(AwsTaskTranslation {}),
    ));

    println!("Syncing {} lambdas", lambda_fns.len());
    let mut tasks = Vec::new();
    for lambda_fn in &lambda_fns {
        tasks.push(
            task_pool.lambda_task_with_reply(
                LambdaTaskKind::CreateFunction,
                lambda_fn.route_key.clone(),
            )?,
        );
    }
    for task in tasks {
        task.await?;
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    // let removing = deployed_state.collect_deployed_components(&project.name);
    // if !removing.is_empty() {
    //     println!("Removing {} lambdas", removing.len());
    //     for components in removing {
    //         sync_tasks.push(RemoveFn(Box::new(RemoveFnParams {
    //             api_id: project.aws.api.id.clone(),
    //             components,
    //         })))
    //     }
    // }

    println!("\nLambdas deployed to API Gateway\n---");

    for lambda_fn in lambda_fns {
        println!(
            "{} https://{}.execute-api.{}.amazonaws.com/development/{}",
            lambda_fn.route_key.http_method,
            &aws.api.id,
            aws.sdk_clients.region(),
            lambda_fn.route_key.http_path,
        );
    }

    Ok(())
}
