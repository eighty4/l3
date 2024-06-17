use crate::aws::lambda::FunctionArn;
use aws_sdk_apigatewayv2::types::{IntegrationType, ProtocolType};

use crate::lambda::HttpMethod;

pub async fn create_api(
    api_gateway: &aws_sdk_apigatewayv2::Client,
    project_name: &String,
) -> Result<String, anyhow::Error> {
    let api_result = api_gateway
        .create_api()
        .name(format!("l3-{project_name}-api"))
        .description(format!("L3 API for project {project_name}"))
        .protocol_type(ProtocolType::Http)
        .send()
        .await?;
    let api_id = api_result.api_id.unwrap();
    create_stage(api_gateway, api_id.clone()).await?;
    Ok(api_id)
}

async fn create_stage(
    api_gateway: &aws_sdk_apigatewayv2::Client,
    api_id: String,
) -> Result<(), anyhow::Error> {
    api_gateway
        .create_stage()
        .api_id(api_id)
        .stage_name("development")
        .auto_deploy(true)
        .send()
        .await?;
    Ok(())
}

pub async fn add_fn_to_api(
    api_gateway: &aws_sdk_apigatewayv2::Client,
    api_id: String,
    fn_arn: FunctionArn,
    method: HttpMethod,
    path: String,
) -> Result<(), anyhow::Error> {
    let integration_id = create_integration(api_gateway, api_id.clone(), fn_arn).await?;
    create_route(api_gateway, api_id, integration_id, method, path).await?;
    Ok(())
}

async fn create_integration(
    api_gateway: &aws_sdk_apigatewayv2::Client,
    api_id: String,
    fn_arn: FunctionArn,
) -> Result<String, anyhow::Error> {
    Ok(api_gateway
        .create_integration()
        .api_id(api_id)
        .integration_type(IntegrationType::AwsProxy)
        .integration_uri(fn_arn)
        .payload_format_version("2.0")
        .send()
        .await?
        .integration_id
        .unwrap())
}

async fn create_route(
    api_gateway: &aws_sdk_apigatewayv2::Client,
    api_id: String,
    integration_id: String,
    api_method: HttpMethod,
    api_path: String,
) -> Result<(), anyhow::Error> {
    api_gateway
        .create_route()
        .api_id(api_id)
        .route_key(format!("{} /{}", api_method, api_path))
        .target(format!("integrations/{integration_id}"))
        .send()
        .await?;
    Ok(())
}
