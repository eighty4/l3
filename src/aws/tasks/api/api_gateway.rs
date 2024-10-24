use crate::aws::clients::AwsClients;
use crate::aws::resources::{AwsGatewayIntegration, AwsGatewayRoute, FunctionArn, IntegrationId};
use crate::aws::AwsProject;
use crate::lambda::{LambdaFn, RouteKey};
use anyhow::anyhow;
use aws_sdk_apigatewayv2::types::IntegrationType;
use std::sync::Arc;

pub async fn create_route(
    sdk_clients: &AwsClients,
    api_id: &String,
    route_key: &RouteKey,
    integration_id: &IntegrationId,
) -> Result<Arc<AwsGatewayRoute>, anyhow::Error> {
    Ok(Arc::new(AwsGatewayRoute::from(
        sdk_clients
            .api_gateway
            .create_route()
            .api_id(api_id)
            .route_key(route_key.to_route_key_string())
            .target(format!("integrations/{integration_id}"))
            .send()
            .await
            .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?,
    )))
}

pub async fn create_integration(
    sdk_clients: &AwsClients,
    api_id: &String,
    fn_arn: &String,
) -> Result<Arc<AwsGatewayIntegration>, anyhow::Error> {
    Ok(Arc::new(AwsGatewayIntegration::from(
        sdk_clients
            .api_gateway
            .create_integration()
            .api_id(api_id)
            .integration_type(IntegrationType::AwsProxy)
            .integration_uri(fn_arn)
            .payload_format_version("2.0")
            .send()
            .await
            .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?,
    )))
}

pub async fn update_route_target(
    sdk_clients: &AwsClients,
    api_id: &String,
    route_id: &String,
    integration_id: &String,
) -> Result<Arc<AwsGatewayRoute>, anyhow::Error> {
    Ok(Arc::new(AwsGatewayRoute::from(
        sdk_clients
            .api_gateway
            .update_route()
            .api_id(api_id)
            .route_id(route_id)
            .target(format!("integrations/{integration_id}"))
            .send()
            .await
            .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?,
    )))
}

pub async fn update_integration_uri(
    sdk_clients: &AwsClients,
    api_id: &String,
    integration_id: &String,
    integration_uri: &String,
) -> Result<Arc<AwsGatewayIntegration>, anyhow::Error> {
    Ok(Arc::new(AwsGatewayIntegration::from(
        sdk_clients
            .api_gateway
            .update_integration()
            .api_id(api_id)
            .integration_id(integration_id)
            .integration_uri(integration_uri)
            .send()
            .await
            .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?,
    )))
}

pub async fn does_api_gateway_have_invoke_permission(
    aws: &Arc<AwsProject>,
    lambda_fn: &Arc<LambdaFn>,
) -> Result<bool, anyhow::Error> {
    match aws
        .sdk_clients
        .lambda
        .get_policy()
        .function_name(&lambda_fn.fn_name)
        .send()
        .await
    {
        Ok(get_policy_output) => Ok(get_policy_output
            .policy
            .unwrap()
            .contains(format!("\"Sid\":\"{}_{}\"", aws.api.id, aws.api.stage_name).as_str())),
        Err(err) => {
            let service_err = err.into_service_error();
            if service_err.is_resource_not_found_exception() {
                Ok(false)
            } else {
                Err(anyhow!("{}", service_err.to_string()))
            }
        }
    }
}

pub async fn add_api_gateway_invoke_permission(
    aws: &Arc<AwsProject>,
    lambda_fn: &Arc<LambdaFn>,
    fn_arn: &FunctionArn,
) -> Result<(), anyhow::Error> {
    let source_arn = format!(
        "arn:aws:execute-api:{}:{}:{}/{}/{}/{}",
        aws.sdk_clients.region(),
        aws.account_id,
        aws.api.id,
        aws.api.stage_name,
        lambda_fn.route_key.http_method,
        lambda_fn.route_key.http_path,
    );
    aws.sdk_clients
        .lambda
        .add_permission()
        .statement_id(format!("{}_{}", aws.api.id, aws.api.stage_name))
        .function_name(fn_arn)
        .action("lambda:InvokeFunction")
        .principal("apigateway.amazonaws.com")
        .source_arn(source_arn)
        .send()
        .await
        .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?;
    Ok(())
}
