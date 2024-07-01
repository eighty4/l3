use std::fs;
use std::path::PathBuf;

use anyhow::anyhow;
use aws_sdk_apigatewayv2::operation::update_integration::UpdateIntegrationOutput;
use aws_sdk_apigatewayv2::operation::update_route::UpdateRouteOutput;
use aws_sdk_apigatewayv2::types::IntegrationType;
use aws_sdk_iam::primitives::Blob;
use aws_sdk_lambda::types::Environment;

use crate::aws::clients::AwsClients;
use crate::aws::lambda::{FunctionArn, IntegrationId, RouteId};
use crate::aws::operations::lambda::{
    create_fn, wait_for_fn_state_active, wait_for_fn_update_successful,
};
use crate::aws::tasks::DeployFnParams;
use crate::code::archiver::Archiver;
use crate::code::checksum::ChecksumTree;
use crate::lambda::RouteKey;

pub async fn perform_deploy_fn(
    sdk_clients: &AwsClients,
    params: &DeployFnParams,
) -> Result<(), anyhow::Error> {
    // todo updating code should upload to s3
    // todo include imported deps in archive
    // todo skip update code if checksums are clean
    fs::create_dir_all(
        PathBuf::from(".l3")
            .join(&params.api_id)
            .join(&params.lambda_fn.fn_name),
    )?;
    let mut checksums = ChecksumTree::new(
        params.project_dir.clone(),
        &params.api_id,
        &params.lambda_fn.fn_name,
    )?;
    let env_vars = params.lambda_fn.env_var_sources.read_env_variables()?;
    let synced_fn_arn = match &params.components.function_arn {
        None => {
            let archive_path = create_archive(params)?;
            let created_fn_arn = create_fn(
                &sdk_clients.lambda,
                &params.lambda_fn.fn_name,
                &archive_path,
                &params.lambda_fn.handler_path(),
                &params.lambda_role_arn,
                env_vars,
            )
            .await?;
            add_api_gateway_invoke_permission(sdk_clients, params, &created_fn_arn).await?;
            checksums.update_checksum(params.lambda_fn.source_file.path.clone())?;
            checksums.update_env_var_checksums(&params.lambda_fn.env_var_sources)?;
            println!("  ✔ Created Lambda Function {}", &params.lambda_fn.fn_name);
            wait_for_fn_state_active(&sdk_clients.lambda, &params.lambda_fn.fn_name).await?;
            created_fn_arn
        }
        Some(updating_fn_arn) => {
            if !does_api_gateway_have_invoke_permission(sdk_clients, params).await? {
                add_api_gateway_invoke_permission(sdk_clients, params, updating_fn_arn).await?;
            }
            if !checksums.do_env_checksums_match(&params.lambda_fn.env_var_sources)? {
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
                checksums.update_env_var_checksums(&params.lambda_fn.env_var_sources)?;
                println!(
                    "  ✔ Updated env vars for Lambda Function {}",
                    &params.lambda_fn.fn_name
                );
            }
            if checksums.do_checksums_match(&params.lambda_fn.source_file.path)? {
                println!(
                    "  ✔ Lambda {} code already up to date!",
                    &params.lambda_fn.fn_name
                );
            } else {
                let archive_path = create_archive(params)?;
                sdk_clients
                    .lambda
                    .update_function_code()
                    .function_name(&params.lambda_fn.fn_name)
                    .zip_file(Blob::new(fs::read(&archive_path)?))
                    .send()
                    .await
                    .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?;
                checksums.update_checksum(params.lambda_fn.source_file.path.clone())?;
                // todo wait for publish to finish
                println!(
                    "  ✔ Updated code for Lambda Function {}",
                    &params.lambda_fn.fn_name
                );
            }
            updating_fn_arn.clone()
        }
    };

    // todo publish version and use fully qualified published version arn for api gateway
    if params.publish_fn_updates {
        println!("todo: do not forget to `aws lambda publish-version`")
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

async fn does_api_gateway_have_invoke_permission(
    sdk_clients: &AwsClients,
    params: &DeployFnParams,
) -> Result<bool, anyhow::Error> {
    match sdk_clients
        .lambda
        .get_policy()
        .function_name(&params.lambda_fn.fn_name)
        .send()
        .await
    {
        Ok(get_policy_output) => Ok(get_policy_output
            .policy
            .unwrap()
            .contains(format!("\"Sid\":\"{}_{}\"", params.api_id, params.stage_name).as_str())),
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

async fn add_api_gateway_invoke_permission(
    sdk_clients: &AwsClients,
    params: &DeployFnParams,
    fn_arn: &FunctionArn,
) -> Result<(), anyhow::Error> {
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
        .function_name(fn_arn)
        .action("lambda:InvokeFunction")
        .principal("apigateway.amazonaws.com")
        .source_arn(source_arn)
        .send()
        .await
        .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?;
    Ok(())
}

fn create_archive(params: &DeployFnParams) -> Result<PathBuf, anyhow::Error> {
    let archiver = Archiver::new(
        params.project_dir.clone(),
        PathBuf::from(".l3")
            .join(&params.api_id)
            .join(&params.lambda_fn.fn_name)
            .join("archive.zip")
            .to_path_buf(),
        vec![params.lambda_fn.source_file.path.clone()],
    );
    let p = archiver.write()?;
    println!(
        "  ✔ Built code archive for Lambda Function {}",
        params.lambda_fn.fn_name
    );
    Ok(p)
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
