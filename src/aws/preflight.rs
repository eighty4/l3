use anyhow::anyhow;
use aws_sdk_apigatewayv2::types::ProtocolType;
use aws_sdk_iam::types::Role;
use std::path::Path;
use std::sync::Arc;

use crate::aws::clients::AwsClients;
use crate::aws::resources::repository::AwsResources;
use crate::aws::{AwsApiGateway, AwsApiGatewayConfig, AwsDataDir, DEFAULT_STAGE_NAME};

pub struct AwsPreflightData {
    pub account_id: String,
    pub api: Arc<AwsApiGateway>,
    pub lambda_role: Role,
    pub resources: Arc<AwsResources>,
    pub sdk_clients: Arc<AwsClients>,
}

impl AwsPreflightData {
    pub async fn initialize(
        api_config: &AwsApiGatewayConfig,
        project_dir: &Path,
        project_name: &String,
    ) -> Result<Self, anyhow::Error> {
        let sdk_clients = AwsClients::new(project_name).await?;
        sdk_clients.expect_credentials().await?;
        let api =
            validate_or_create_api(api_config, project_dir, project_name, &sdk_clients).await?;
        let account_id = get_account_id(&sdk_clients.iam).await?;
        let lambda_role = create_lambda_role(&sdk_clients.iam, project_name).await?;
        let resources = AwsResources::new(api.clone(), project_name.clone(), sdk_clients.clone());
        Ok(Self {
            account_id,
            api,
            lambda_role,
            resources,
            sdk_clients,
        })
    }
}

async fn validate_or_create_api(
    api_config: &AwsApiGatewayConfig,
    project_dir: &Path,
    project_name: &String,
    sdk_clients: &AwsClients,
) -> Result<Arc<AwsApiGateway>, anyhow::Error> {
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
    Ok(AwsApiGateway::new(id, stage_name))
}

async fn create_api(
    api_gateway: &aws_sdk_apigatewayv2::Client,
    project_name: &String,
    stage_name: &String,
) -> Result<String, anyhow::Error> {
    let create_api_output = api_gateway
        .create_api()
        .name(format!("l3-{project_name}-api"))
        .description(format!("L3 API for project {project_name}"))
        .protocol_type(ProtocolType::Http)
        .send()
        .await?;
    let api_id = create_api_output.api_id.unwrap();
    api_gateway
        .create_stage()
        .api_id(&api_id)
        .stage_name(stage_name)
        .auto_deploy(true)
        .send()
        .await?;
    Ok(api_id)
}

async fn does_api_exist(
    api_gateway: &aws_sdk_apigatewayv2::Client,
    api_id: &String,
) -> Result<bool, anyhow::Error> {
    match api_gateway.get_api().api_id(api_id).send().await {
        Ok(_) => Ok(true),
        Err(err) => {
            let service_error = err.as_service_error();
            if service_error.is_some() && service_error.unwrap().is_not_found_exception() {
                Ok(false)
            } else {
                Err(anyhow!("does_api_exist error from aws sdk {}", err))
            }
        }
    }
}

async fn get_account_id(iam: &aws_sdk_iam::Client) -> Result<String, anyhow::Error> {
    let user_arn = iam.get_user().send().await?.user.unwrap().arn;
    Ok(user_arn.split(':').nth(4).unwrap().to_string())
}

async fn create_lambda_role(
    iam: &aws_sdk_iam::Client,
    project_name: &String,
) -> Result<Role, anyhow::Error> {
    let role_name = format!("l3-{project_name}-lambda-role");
    let mut role = get_role(iam, &role_name).await?;
    if role.is_none() {
        role = iam
            .create_role()
            .role_name(&role_name)
            .assume_role_policy_document(include_str!("l3-trust.json"))
            .send()
            .await?
            .role;
    }
    // todo check if attach-role_policy is idempotent?
    iam.attach_role_policy()
        .role_name(&role_name)
        .policy_arn("arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole")
        .send()
        .await?;
    Ok(role.unwrap())
}

async fn get_role(
    iam: &aws_sdk_iam::Client,
    role_name: &String,
) -> Result<Option<Role>, anyhow::Error> {
    match iam.get_role().role_name(role_name).send().await {
        Ok(get_role_output) => Ok(get_role_output.role),
        Err(err) => match err.as_service_error() {
            None => Err(anyhow!(err)),
            Some(service_error) => {
                if service_error.is_no_such_entity_exception() {
                    Ok(None)
                } else {
                    Err(anyhow!(err))
                }
            }
        },
    }
}
