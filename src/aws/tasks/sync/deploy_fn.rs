use anyhow::anyhow;
use aws_sdk_iam::primitives::Blob;
use aws_sdk_lambda::types::Environment;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::aws::clients::AwsClients;
use crate::aws::state::DeployedLambdaComponents;
use crate::aws::tasks::api::api_gateway::{
    add_api_gateway_invoke_permission, create_integration, create_route,
    does_api_gateway_have_invoke_permission, update_integration_uri, update_route_target,
};
use crate::aws::tasks::api::lambda::{
    create_fn_w_retry_for_role_not_ready, wait_for_fn_state_active, wait_for_fn_update_successful,
};
use crate::code::build::LambdaFnBuild;
use crate::code::checksum::ChecksumTree;
use crate::lambda::LambdaFn;
use crate::project::Lx3ProjectDeets;

pub struct DeployFnParams {
    pub components: DeployedLambdaComponents,
    pub lambda_fn: Arc<LambdaFn>,
    pub project_deets: Arc<Lx3ProjectDeets>,
    pub publish_fn_updates: bool,
}

/// Super task for bringing up a Lambda and API Gateway integration. Used by `l3 sync` command that
/// schedules this task for any local source function regardless of deployed remote state. This task
/// diffs environment and sources checksums and API Gateway connectivity to determine what AWS API
/// calls need to be performed to deploy the function.
pub async fn perform_deploy_fn(
    sdk_clients: &AwsClients,
    params: &DeployFnParams,
) -> Result<(), anyhow::Error> {
    // todo updating code should upload to s3 to avoid upload churn on retries
    // todo include imported deps in archive
    fs::create_dir_all(
        PathBuf::from(".l3")
            .join("aws")
            .join(&params.project_deets.aws.api.id)
            .join(&params.lambda_fn.fn_name),
    )?;
    let mut checksums = ChecksumTree::new(
        params.project_deets.project_dir.clone(),
        &params.project_deets.aws.api.id,
        &params.lambda_fn.fn_name,
    )
    .await?;
    let env_vars = params.lambda_fn.env_var_sources.read_env_variables()?;
    let synced_fn_arn = match &params.components.function_arn {
        None => {
            let archive_path = build_and_zip_sources(params).await?;
            let created_fn_arn = create_fn_w_retry_for_role_not_ready(
                &sdk_clients.lambda,
                &params.lambda_fn.fn_name,
                &archive_path,
                &params.lambda_fn.handler_path(),
                &params.project_deets.aws.lambda_role.arn,
                env_vars,
            )
            .await?;
            add_api_gateway_invoke_permission(
                &params.project_deets,
                &params.lambda_fn,
                &created_fn_arn,
            )
            .await?;
            checksums.update_checksum(params.lambda_fn.path.rel.clone())?;
            checksums.update_env_var_checksums(&params.lambda_fn.env_var_sources)?;
            println!("  ✔ Created Lambda Function {}", &params.lambda_fn.fn_name);
            wait_for_fn_state_active(&sdk_clients.lambda, &params.lambda_fn.fn_name).await?;
            created_fn_arn
        }
        Some(updating_fn_arn) => {
            if !does_api_gateway_have_invoke_permission(&params.project_deets, &params.lambda_fn)
                .await?
            {
                add_api_gateway_invoke_permission(
                    &params.project_deets,
                    &params.lambda_fn,
                    updating_fn_arn,
                )
                .await?;
            }
            if !checksums.do_env_checksums_match(
                &params.components.function_env,
                &params.lambda_fn.env_var_sources,
            )? {
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
            if checksums.do_checksums_match(&params.lambda_fn.path.rel)? {
                println!(
                    "  ✔ Lambda {} code already up to date!",
                    &params.lambda_fn.fn_name
                );
            } else {
                let archive_path = build_and_zip_sources(params).await?;
                sdk_clients
                    .lambda
                    .update_function_code()
                    .function_name(&params.lambda_fn.fn_name)
                    .zip_file(Blob::new(fs::read(&archive_path)?))
                    .send()
                    .await
                    .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?;
                checksums.update_checksum(params.lambda_fn.path.rel.clone())?;
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
            let integration_id = create_integration(
                sdk_clients,
                &params.project_deets.aws.api.id,
                &synced_fn_arn,
            )
            .await?;
            println!(
                "  ✔ Created API Gateway Integration for {}",
                &params.lambda_fn.fn_name
            );
            create_route(
                sdk_clients,
                &params.project_deets.aws.api.id,
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
                let integration_id = create_integration(
                    sdk_clients,
                    &params.project_deets.aws.api.id,
                    &synced_fn_arn,
                )
                .await?;
                println!(
                    "  ✔ Created API Gateway Integration for {}",
                    &params.lambda_fn.fn_name
                );
                update_route_target(
                    sdk_clients,
                    &params.project_deets.aws.api.id,
                    route_id,
                    &integration_id,
                )
                .await?;
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
                        &params.project_deets.aws.api.id,
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

async fn build_and_zip_sources(params: &DeployFnParams) -> Result<PathBuf, anyhow::Error> {
    let archive = LambdaFnBuild::new(params.lambda_fn.clone(), params.project_deets.clone())
        .create_code_archive()
        .await?;
    println!(
        "  ✔ Built code archive for Lambda Function {}",
        params.lambda_fn.fn_name
    );
    Ok(archive.path)
}
