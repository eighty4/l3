use std::fs;
use std::path::PathBuf;

use anyhow::anyhow;
use aws_sdk_apigatewayv2::operation::update_integration::UpdateIntegrationOutput;
use aws_sdk_apigatewayv2::operation::update_route::UpdateRouteOutput;
use aws_sdk_apigatewayv2::types::IntegrationType;
use aws_sdk_iam::primitives::Blob;
use aws_sdk_lambda::types::Environment;

use crate::aws::clients::AwsClients;
use crate::aws::lambda::{IntegrationId, RouteId};
use crate::aws::operations::lambda::{
    create_fn, wait_for_fn_state_active, wait_for_fn_update_successful,
};
use crate::aws::tasks::DeployFnParams;
use crate::code::archive::CODE_ARCHIVE_PATH;
use crate::lambda::RouteKey;

pub async fn perform_deploy_fn(
    sdk_clients: &AwsClients,
    params: &DeployFnParams,
) -> Result<(), anyhow::Error> {
    let env_vars = params.lambda_fn.env_var_sources.read_env_variables()?;
    let mut updated_env_vars = true;
    let synced_fn_arn = match &params.components.function_arn {
        None => {
            let created_fn_arn = create_fn(
                &sdk_clients.lambda,
                &params.lambda_fn.fn_name,
                &PathBuf::from(CODE_ARCHIVE_PATH),
                &params.lambda_fn.handler_path(),
                &params.lambda_role_arn,
                env_vars,
            )
            .await?;
            println!("  ✔ Created Lambda Function {}", &params.lambda_fn.fn_name);
            wait_for_fn_state_active(&sdk_clients.lambda, &params.lambda_fn.fn_name).await?;

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
            created_fn_arn
        }
        Some(updating_fn_arn) => {
            if !params
                .lambda_fn
                .env_var_sources
                .requires_update(&params.api_id)?
            {
                updated_env_vars = false;
            } else {
                sdk_clients
                    .lambda
                    .update_function_configuration()
                    .function_name(&params.lambda_fn.fn_name)
                    .environment(Environment::builder().set_variables(env_vars).build())
                    .send()
                    .await
                    .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?;
                wait_for_fn_update_successful(&sdk_clients.lambda, &params.lambda_fn.fn_name)
                    .await?;
                println!(
                    "  ✔ Updated env vars for Lambda Function {}",
                    &params.lambda_fn.fn_name
                );
            }
            sdk_clients
                .lambda
                .update_function_code()
                .function_name(&params.lambda_fn.fn_name)
                .zip_file(Blob::new(fs::read(&PathBuf::from(CODE_ARCHIVE_PATH))?))
                .send()
                .await
                .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?;
            // todo update code
            // todo wait for publish to finish
            println!(
                "  ✔ Updated code for Lambda Function {}",
                &params.lambda_fn.fn_name
            );
            updating_fn_arn.clone()
        }
    };

    // todo publish version and use fully qualified published version arn for api gateway
    if params.publish_fn_updates {
        println!("todo: do not forget to `aws lambda publish-version`")
    }

    if updated_env_vars {
        params
            .lambda_fn
            .env_var_sources
            .update_cached_checksums(&params.api_id)?;
    }

    match &params.components.route {
        None => {
            let integration_id =
                create_integration(sdk_clients, &params.api_id, &synced_fn_arn).await?;
            println!(
                "  ✔ Created API Gateway Integration for {}",
                &params.lambda_fn.fn_name
            );
            create_route(
                sdk_clients,
                &params.api_id,
                &params.lambda_fn.route_key,
                &integration_id,
            )
            .await?;
            println!(
                "  ✔ Created API Gateway Route for {} to call {}",
                &params.lambda_fn.route_key.to_route_key_string(),
                &params.lambda_fn.fn_name
            );
        }
        Some(route_id) => match &params.components.integration {
            None => {
                let integration_id =
                    create_integration(sdk_clients, &params.api_id, &synced_fn_arn).await?;
                println!(
                    "  ✔ Created API Gateway Integration for {}",
                    &params.lambda_fn.fn_name
                );
                update_route_target(sdk_clients, &params.api_id, route_id, &integration_id).await?;
                println!(
                    "  ✔ Updated API Gateway Route for {} to call {}",
                    &params.lambda_fn.route_key.to_route_key_string(),
                    &params.lambda_fn.fn_name
                );
            }
            Some((integration_id, integration_uri)) => {
                if integration_uri.as_str() != synced_fn_arn.as_str() {
                    update_integration_uri(
                        sdk_clients,
                        &params.api_id,
                        integration_id,
                        &synced_fn_arn,
                    )
                    .await?;
                    println!(
                        "  ✔ Updated API Gateway Integration for {} to call {}",
                        &params.lambda_fn.route_key.to_route_key_string(),
                        &params.lambda_fn.fn_name
                    );
                }
            }
        },
    };
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
