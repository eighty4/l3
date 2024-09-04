use crate::aws::preflight::AwsPreflightData;
use crate::aws::{AwsApiConfig, AwsDataDir, AwsDeets};
use crate::code::runtime::RuntimeConfig;
use crate::code::source::tracker::SourceTracker;
use crate::code::source::tree::SourceTree;
use crate::notification::{LambdaEvent, LambdaEventKind, LambdaNotification, LambdaUpdateResult};
use crate::project::Lx3ProjectDeets;
use crate::ui::confirm::confirm;
use crate::ui::dev::print_notification;
use crate::ui::exit::exit;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;

pub struct DevOptions {
    pub aws: AwsApiConfig,
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
    let project_deets = Arc::new(
        Lx3ProjectDeets::builder()
            .aws_deets(AwsDeets::from(aws_preflight_data))
            .runtime_config(runtime_config)
            .build(
                dev_options.project_dir.clone(),
                dev_options.project_name.clone(),
            ),
    );

    println!("λλλ dev");
    println!("  project: {}", &project_deets.project_name);
    println!("  region: {}", &project_deets.aws.region);
    println!("  api id: {}", &project_deets.aws.api.id);
    println!();

    if !dev_options.auto_confirm && !confirm("\n  Continue with syncing?")? {
        println!("  Cancelling sync operations!");
        process::exit(0);
    }

    if dev_options.clear_cache {
        println!(
            "\nClearing cache at .l3/aws/{} and re-syncing",
            &project_deets.aws.api.id
        );
        AwsDataDir::clear_cache(&project_deets.aws.api.id, &project_deets.project_dir);
    }

    AwsDataDir::cache_api_id(&project_deets.project_dir, &project_deets.aws.api.id)?;

    let (notification_tx, mut notification_rx) = unbounded_channel::<LambdaNotification>();
    let (source_tree, sources_api) =
        SourceTree::new(notification_tx.clone(), project_deets.clone());
    let source_tracker = SourceTracker::new(project_deets, runtime_config_api, sources_api.clone());
    sources_api.refresh_routes().await?;

    loop {
        match notification_rx.recv().await {
            None => break,
            Some(notification) => {
                print_notification(&notification);
                if let LambdaNotification::Lambda(LambdaEvent {
                    kind:
                        LambdaEventKind::Created(LambdaUpdateResult::Failure(_))
                        | LambdaEventKind::Removed(LambdaUpdateResult::Failure(_))
                        | LambdaEventKind::Updated(_, LambdaUpdateResult::Failure(_)),
                    ..
                }) = notification
                {
                    exit(1);
                }
            }
        }
    }
    Ok(())
}
