use std::path::PathBuf;
use std::sync::Arc;

use crate::aws::preflight::AwsPreflightData;
use crate::aws::{AwsApiConfig, AwsDeets};
use crate::code::runtime::SourcesRuntimeDeets;
use crate::project::Lx3ProjectDeets;

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

    Ok(())
}
