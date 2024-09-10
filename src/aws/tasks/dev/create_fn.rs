use crate::aws::state::DeployedProjectState;
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
use crate::notification::{LambdaEventKind, LambdaUpdateResult};
use crate::project::Lx3ProjectDeets;
use crate::task::launch::LaunchedTask;
use anyhow::anyhow;
use aws_sdk_iam::primitives::Blob;
use aws_sdk_lambda::types::Environment;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub fn create_fn(project_deets: Arc<Lx3ProjectDeets>, lambda_fn: Arc<LambdaFn>) -> LaunchedTask {
    Box::pin(create_fn_inner(project_deets, lambda_fn))
}

async fn create_fn_inner(
    project_deets: Arc<Lx3ProjectDeets>,
    lambda_fn: Arc<LambdaFn>,
) -> Result<(), anyhow::Error> {
    project_deets.send_lambda_event(lambda_fn.clone(), LambdaEventKind::Creating);
    let result =
        match create_api_gateway_and_lambda_fn_resources(&project_deets, lambda_fn.clone()).await {
            Ok(_) => LambdaUpdateResult::Success,
            Err(err) => LambdaUpdateResult::Failure(err.to_string()),
        };
    project_deets.send_lambda_event(lambda_fn.clone(), LambdaEventKind::Created(result));
    Ok(())
}

async fn create_api_gateway_and_lambda_fn_resources(
    project_deets: &Arc<Lx3ProjectDeets>,
    lambda_fn: Arc<LambdaFn>,
) -> Result<(), anyhow::Error> {
    let mut project_state = DeployedProjectState::fetch_from_aws(
        &project_deets.aws.sdk_clients,
        &project_deets.project_name,
        &project_deets.aws.api.id,
    )
    .await?;
    let components = project_state.rm_deployed_components(&lambda_fn.route_key, &lambda_fn.fn_name);
    fs::create_dir_all(
        PathBuf::from(".l3")
            .join("aws")
            .join(&project_deets.aws.api.id)
            .join(&lambda_fn.fn_name),
    )?;
    let mut checksums = ChecksumTree::new(
        project_deets.project_dir.clone(),
        &project_deets.aws.api.id,
        &lambda_fn.fn_name,
    )
    .await?;
    let env_vars = lambda_fn.env_var_sources.read_env_variables()?;
    let synced_fn_arn = match &components.function_arn {
        None => {
            let archive_path =
                build_and_zip_sources(lambda_fn.clone(), project_deets.clone()).await?;
            let created_fn_arn = create_fn_w_retry_for_role_not_ready(
                &project_deets.aws.sdk_clients.lambda,
                &lambda_fn.fn_name,
                &archive_path,
                &lambda_fn.handler_path(),
                &project_deets.aws.lambda_role.arn,
                env_vars,
            )
            .await?;
            add_api_gateway_invoke_permission(project_deets, &lambda_fn, &created_fn_arn).await?;
            checksums.update_checksum(lambda_fn.path.rel.clone())?;
            checksums.update_env_var_checksums(&lambda_fn.env_var_sources)?;
            wait_for_fn_state_active(&project_deets.aws.sdk_clients.lambda, &lambda_fn.fn_name)
                .await?;
            created_fn_arn
        }
        Some(updating_fn_arn) => {
            if !does_api_gateway_have_invoke_permission(project_deets, &lambda_fn).await? {
                add_api_gateway_invoke_permission(project_deets, &lambda_fn, updating_fn_arn)
                    .await?;
            }
            if !checksums
                .do_env_checksums_match(&components.function_env, &lambda_fn.env_var_sources)?
            {
                project_deets
                    .aws
                    .sdk_clients
                    .lambda
                    .update_function_configuration()
                    .function_name(&lambda_fn.fn_name)
                    .environment(Environment::builder().set_variables(env_vars).build())
                    .send()
                    .await
                    .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?;
                wait_for_fn_update_successful(
                    &project_deets.aws.sdk_clients.lambda,
                    &lambda_fn.fn_name,
                )
                .await?;
                checksums.update_env_var_checksums(&lambda_fn.env_var_sources)?;
            }
            if !checksums.do_checksums_match(&lambda_fn.path.rel)? {
                let archive_path =
                    build_and_zip_sources(lambda_fn.clone(), project_deets.clone()).await?;
                project_deets
                    .aws
                    .sdk_clients
                    .lambda
                    .update_function_code()
                    .function_name(&lambda_fn.fn_name)
                    .zip_file(Blob::new(fs::read(&archive_path)?))
                    .send()
                    .await
                    .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?;
                checksums.update_checksum(lambda_fn.path.rel.clone())?;
                // todo wait for publish to finish
            }
            updating_fn_arn.clone()
        }
    };

    match &components.route {
        None => {
            let integration_id = create_integration(
                &project_deets.aws.sdk_clients,
                &project_deets.aws.api.id,
                &synced_fn_arn,
            )
            .await?;
            create_route(
                &project_deets.aws.sdk_clients,
                &project_deets.aws.api.id,
                &lambda_fn.route_key,
                &integration_id,
            )
            .await?;
        }
        Some(route_id) => match &components.integration {
            None => {
                let integration_id = create_integration(
                    &project_deets.aws.sdk_clients,
                    &project_deets.aws.api.id,
                    &synced_fn_arn,
                )
                .await?;
                update_route_target(
                    &project_deets.aws.sdk_clients,
                    &project_deets.aws.api.id,
                    route_id,
                    &integration_id,
                )
                .await?;
            }
            Some((integration_id, integration_uri)) => {
                if integration_uri.as_str() != synced_fn_arn.as_str() {
                    update_integration_uri(
                        &project_deets.aws.sdk_clients,
                        &project_deets.aws.api.id,
                        integration_id,
                        &synced_fn_arn,
                    )
                    .await?;
                }
            }
        },
    };
    Ok(())
}

async fn build_and_zip_sources(
    lambda_fn: Arc<LambdaFn>,
    project_deets: Arc<Lx3ProjectDeets>,
) -> Result<PathBuf, anyhow::Error> {
    let archive = LambdaFnBuild::new(lambda_fn, project_deets)
        .create_code_archive()
        .await?;
    Ok(archive.path)
}
