use std::path::PathBuf;
use std::process;
use std::sync::Arc;

use crate::aws::preflight::AwsPreflightData;
use crate::aws::{AwsApiConfig, AwsDataDir, AwsDeets};
use crate::code::runtime::SourcesRuntimeDeets;
use crate::code::source::tracker::SourceTracker;
use crate::code::source::tree::SourceTree;
use crate::project::Lx3ProjectDeets;
use crate::ui::confirm::confirm;
// use crate::task::executor::TaskExecutor;
use crate::ui::dashboard::TerminalDashboard;

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
    let project_deets = Arc::new(
        Lx3ProjectDeets::builder()
            .aws_deets(AwsDeets::from(aws_preflight_data))
            .runtime_deets(SourcesRuntimeDeets::read_details(&dev_options.project_dir)?)
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

    if !dev_options.auto_confirm && !confirm("  Continue with syncing?")? {
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

    let mut src_tree = SourceTree::new(project_deets.clone());
    src_tree.initialize().await?;
    let dev_ui = TerminalDashboard::new();
    let (_, _) = SourceTracker::new(project_deets, src_tree)?;
    // _ = TaskExecutor::new(dev_ui.de_tx.clone(), src_rx);

    tokio::spawn(async move { dev_ui.start_dev_interface().await }).await?;
    Ok(())
}
