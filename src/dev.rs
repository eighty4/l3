use crate::aws::preflight::AwsPreflightData;
use crate::aws::tasks::AwsTaskTranslation;
use crate::aws::{AwsApiGatewayConfig, AwsDataDir, AwsProject};
use crate::code::runtime::RuntimeConfig;
use crate::code::source::tracker::SourceTracker;
use crate::code::source::tree::SourceTree;
use crate::notification::{LambdaEvent, LambdaEventKind, LambdaNotification, LambdaUpdateResult};
use crate::project::Lx3Project;
use crate::task::executor::TaskExecutor;
use crate::task::pool::TaskPool;
use crate::ui::dev::print_notification;
use crate::ui::exit::err_exit;
use crate::ui::prompt::confirm::prompt_for_confirmation;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

pub struct DevOptions {
    pub aws: AwsApiGatewayConfig,
    pub auto_confirm: bool,
    pub clear_cache: bool,
    pub project_dir: PathBuf,
    pub project_name: String,
}

pub async fn develop_project(dev_options: DevOptions) -> Result<(), anyhow::Error> {
    let aws_preflight_data = AwsPreflightData::initialize(
        &dev_options.aws,
        &dev_options.project_dir,
        &dev_options.project_name,
    )
    .await?;
    let (runtime_config, runtime_config_api) = RuntimeConfig::new(dev_options.project_dir.clone());
    runtime_config_api.initialize_runtime_configs().await;
    let (project, mut notification_rx) = Lx3Project::builder()
        .aws(Arc::new(AwsProject::from(aws_preflight_data)))
        .runtime_config(runtime_config)
        .build(
            dev_options.project_dir.clone(),
            dev_options.project_name.clone(),
        );

    let aws = project.aws();

    println!("λλλ dev (incomplete feature)");
    println!("  project: {}", &project.name);
    println!("  region: {}", aws.sdk_clients.region());
    println!("  api id: {}", aws.api.id);
    println!();

    let (source_tree, sources_api) = SourceTree::new(project.clone());
    let _source_tracker =
        SourceTracker::new(project.clone(), runtime_config_api, sources_api.clone());
    sources_api.refresh_routes().await?;

    {
        let lambda_fns = source_tree.lock().unwrap().lambda_fns();
        println!("{} lambdas", lambda_fns.len());
        for lambda_fn in lambda_fns {
            println!("  {}", lambda_fn.route_key.to_route_key_string());
        }
    }

    if !dev_options.auto_confirm && !prompt_for_confirmation("\n  Start dev mode?") {
        println!("  Cancelling dev mode!");
        process::exit(0);
    }

    if dev_options.clear_cache {
        println!("\nClearing cache at .l3/aws/{} and re-syncing", &aws.api.id);
        AwsDataDir::clear_cache(&aws.api.id, &project.dir);
    }

    AwsDataDir::cache_api_id(&project.dir, &aws.api.id)?;

    aws.resources.refresh_state().await?;

    let _task_pool = TaskPool::new(TaskExecutor::new(
        project,
        source_tree,
        Box::new(AwsTaskTranslation {}),
    ));

    loop {
        match notification_rx.recv().await {
            None => break,
            Some(notification) => {
                print_notification(&notification);
                exit_on_task_failure(&notification);
            }
        }
    }
    Ok(())
}

fn exit_on_task_failure(notification: &LambdaNotification) {
    if let LambdaNotification::Lambda(LambdaEvent {
        kind:
            LambdaEventKind::Created(LambdaUpdateResult::Failure(_))
            | LambdaEventKind::Removed(LambdaUpdateResult::Failure(_))
            | LambdaEventKind::Updated(_, LambdaUpdateResult::Failure(_)),
        ..
    }) = notification
    {
        err_exit("task failure");
    }
}
