use std::path::PathBuf;

use anyhow::anyhow;
use aws_sdk_apigatewayv2::operation::update_integration::UpdateIntegrationOutput;
use aws_sdk_apigatewayv2::operation::update_route::UpdateRouteOutput;
use aws_sdk_apigatewayv2::types::IntegrationType;

use crate::aws::clients::AwsClients;
use crate::aws::lambda::{IntegrationId, RouteId};
use crate::aws::operations::lambda::create_fn;
use crate::aws::tasks::DeployFnParams;
use crate::code::archive::CODE_ARCHIVE_PATH;
use crate::lambda::RouteKey;

pub async fn perform_deploy_fn(
    sdk_clients: &AwsClients,
    params: &DeployFnParams,
) -> Result<(), anyhow::Error> {
    let fn_name = params.lambda_fn.fn_name(&params.sync_id);
    let env_vars = params.lambda_fn.env_var_sources.read_env_variables()?;
    let created_fn_arn = create_fn(
        &sdk_clients.lambda,
        &fn_name,
        &PathBuf::from(CODE_ARCHIVE_PATH),
        &params.lambda_fn.handler_path(),
        &params.lambda_role_arn,
        env_vars,
    )
    .await?;
    params
        .lambda_fn
        .env_var_sources
        .update_cached_checksums(&params.api_id)?;

    let source_arn = format!(
        "arn:aws:execute-api:{}:{}:{}/{}/{}/{}",
        params.region,
        params.account_id,
        params.api_id,
        params.stage_name,
        params.lambda_fn.route_key.http_method,
        params.lambda_fn.route_key.http_path,
    );
    sdk_clients
        .lambda
        .add_permission()
        .statement_id(format!("{}_{}", params.api_id, params.stage_name))
        .function_name(&created_fn_arn)
        .action("lambda:InvokeFunction")
        .principal("apigateway.amazonaws.com")
        .source_arn(source_arn)
        .send()
        .await
        .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?;

    match &params.deployed_components.route {
        None => {
            // todo create route and integration
            let integration_id =
                create_integration(sdk_clients, &params.api_id, &created_fn_arn).await?;
            create_route(
                sdk_clients,
                &params.api_id,
                &params.lambda_fn.route_key,
                &integration_id,
            )
            .await?;
        }
        Some(route_id) => match &params.deployed_components.integration {
            None => {
                let integration_id =
                    create_integration(sdk_clients, &params.api_id, &created_fn_arn).await?;
                update_route_target(sdk_clients, &params.api_id, route_id, &integration_id).await?;
            }
            Some(integration_id) => {
                update_integration_uri(
                    sdk_clients,
                    &params.api_id,
                    integration_id,
                    &created_fn_arn,
                )
                .await?;
            }
        },
    };
    if let Some(previous_fn_name) = &params.deployed_components.function {
        sdk_clients
            .lambda
            .delete_function()
            .function_name(previous_fn_name)
            .send()
            .await?;
    }
    Ok(())
}

async fn create_route(
    sdk_clients: &AwsClients,
    api_id: &String,
    route_key: &RouteKey,
    integration_id: &IntegrationId,
) -> Result<RouteId, anyhow::Error> {
    Ok(sdk_clients
        .api_gateway
        .create_route()
        .api_id(api_id)
        .route_key(route_key.to_route_key_string())
        .target(format!("integrations/{integration_id}"))
        .send()
        .await
        .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))
        .unwrap()
        .route_id
        .unwrap())
}

async fn create_integration(
    sdk_clients: &AwsClients,
    api_id: &String,
    fn_arn: &String,
) -> Result<IntegrationId, anyhow::Error> {
    Ok(sdk_clients
        .api_gateway
        .create_integration()
        .api_id(api_id)
        .integration_type(IntegrationType::AwsProxy)
        .integration_uri(fn_arn)
        .payload_format_version("2.0")
        .send()
        .await
        .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?
        .integration_id
        .unwrap())
}

async fn update_route_target(
    sdk_clients: &AwsClients,
    api_id: &String,
    route_id: &String,
    integration_id: &String,
) -> Result<UpdateRouteOutput, anyhow::Error> {
    sdk_clients
        .api_gateway
        .update_route()
        .api_id(api_id)
        .route_id(route_id)
        .target(format!("integrations/{integration_id}"))
        .send()
        .await
        .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))
}

async fn update_integration_uri(
    sdk_clients: &AwsClients,
    api_id: &String,
    integration_id: &String,
    integration_uri: &String,
) -> Result<UpdateIntegrationOutput, anyhow::Error> {
    sdk_clients
        .api_gateway
        .update_integration()
        .api_id(api_id)
        .integration_id(integration_id)
        .integration_uri(integration_uri)
        .send()
        .await
        .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))
}
