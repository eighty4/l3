use std::path::Path;

use anyhow::anyhow;
use aws_config::Region;
use aws_sdk_iam::types::Role;

use crate::aws::clients::AwsClients;
use crate::aws::config::load_sdk_config;
use crate::aws::operations::api_gateway::{create_api, does_api_exist};
use crate::aws::operations::iam::{create_lambda_role, get_account_id};
use crate::aws::{AwsApiConfig, AwsApiDeets, AwsDataDir, DEFAULT_STAGE_NAME};

pub struct AwsPreflightData {
    pub account_id: String,
    pub api: AwsApiDeets,
    pub lambda_role: Role,
    pub region: Region,
    pub sdk_clients: AwsClients,
}

impl AwsPreflightData {
    pub async fn initialize(
        api_config: &AwsApiConfig,
        project_dir: &Path,
        project_name: &String,
    ) -> Result<Self, anyhow::Error> {
        let sdk_config = load_sdk_config().await;
        let sdk_clients = AwsClients::from(&sdk_config);
        let api =
            validate_or_create_api(api_config, project_dir, project_name, &sdk_clients).await?;
        let account_id = get_account_id(&sdk_clients.iam).await?;
        let lambda_role = create_lambda_role(&sdk_clients.iam, project_name).await?;
        let region = sdk_config.region().cloned().unwrap();
        Ok(Self {
            account_id,
            api,
            lambda_role,
            region,
            sdk_clients,
        })
    }
}

pub async fn validate_or_create_api(
    api_config: &AwsApiConfig,
    project_dir: &Path,
    project_name: &String,
    sdk_clients: &AwsClients,
) -> Result<AwsApiDeets, anyhow::Error> {
    let maybe_api_id = match &api_config.api_id {
        None => AwsDataDir::read_cached_api_id(project_dir)?,
        Some(api_id) => Some(api_id.clone()),
    };
    let stage_name = match &api_config.stage_name {
        None => DEFAULT_STAGE_NAME.to_string(),
        Some(stage_name) => stage_name.clone(),
    };
    let id = match maybe_api_id {
        None => {
            println!("creating new api gateway");
            create_api(&sdk_clients.api_gateway, project_name, &stage_name).await
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
    }?;
    Ok(AwsApiDeets { id, stage_name })
}
