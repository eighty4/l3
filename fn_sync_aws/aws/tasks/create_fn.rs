use crate::aws::resources::AwsLambdaFunction;
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
use crate::project::Lx3Project;
use crate::task::launch::LaunchedTask;
use anyhow::anyhow;
use aws_sdk_iam::primitives::Blob;
use aws_sdk_lambda::types::Environment;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub fn create_fn(project: Arc<Lx3Project>, lambda_fn: Arc<LambdaFn>) -> LaunchedTask {
    Box::pin(create_fn_inner(project, lambda_fn))
}

async fn create_fn_inner(
    project: Arc<Lx3Project>,
    lambda_fn: Arc<LambdaFn>,
) -> Result<(), anyhow::Error> {
    project.send_lambda_event(lambda_fn.clone(), LambdaEventKind::Creating);
    let result = match create_api_gateway_and_lambda_fn_resources(&project, lambda_fn.clone()).await
    {
        Ok(_) => LambdaUpdateResult::Success,
        Err(err) => LambdaUpdateResult::Failure(err.to_string()),
    };
    project.send_lambda_event(lambda_fn.clone(), LambdaEventKind::Created(result));
    Ok(())
}

async fn create_api_gateway_and_lambda_fn_resources(
    project: &Arc<Lx3Project>,
    lambda_fn: Arc<LambdaFn>,
) -> Result<(), anyhow::Error> {
    let aws = project.aws();
    let components = aws.resources.resources_for_fn(&lambda_fn).await?;
    let _ = fs::create_dir_all(
        PathBuf::from(".l3")
            .join("aws")
            .join(&aws.api.id)
            .join(&lambda_fn.fn_name),
    );
    let mut checksums =
        ChecksumTree::new(project.dir.clone(), &aws.api.id, &lambda_fn.fn_name).await?;
    let env_vars = lambda_fn.env_var_sources.read_env_variables()?;
    let synced_fn = match &components.function {
        None => {
            let build_manifest = LambdaFnBuild::in_api_dir(lambda_fn.clone(), project.clone())
                .build()
                .await?;
            let created_fn = create_fn_w_retry_for_role_not_ready(
                &aws.sdk_clients.lambda,
                &lambda_fn.language,
                &lambda_fn.fn_name,
                &build_manifest.archive_path,
                &lambda_fn.handler_path(),
                &aws.lambda_role.arn,
                env_vars,
            )
            .await?;
            aws.resources.created_lambda_function(created_fn.clone())?;
            add_api_gateway_invoke_permission(&aws, &lambda_fn, &created_fn.arn).await?;
            checksums.update_all_checksums(&build_manifest.fn_sources)?;
            checksums.update_env_var_checksums(&lambda_fn.env_var_sources)?;
            wait_for_fn_state_active(&aws.sdk_clients.lambda, &lambda_fn.fn_name).await?;
            created_fn
        }
        Some(updating_fn) => {
            if !does_api_gateway_have_invoke_permission(&aws, &lambda_fn).await? {
                add_api_gateway_invoke_permission(&aws, &lambda_fn, &updating_fn.arn).await?;
            }
            if !checksums.do_env_checksums_match(&updating_fn.env, &lambda_fn.env_var_sources)? {
                aws.resources
                    .created_lambda_function(Arc::new(AwsLambdaFunction::from(
                        aws.sdk_clients
                            .lambda
                            .update_function_configuration()
                            .function_name(&lambda_fn.fn_name)
                            .environment(Environment::builder().set_variables(env_vars).build())
                            .send()
                            .await
                            .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?,
                    )))?;
                wait_for_fn_update_successful(&aws.sdk_clients.lambda, &lambda_fn.fn_name).await?;
                checksums.update_env_var_checksums(&lambda_fn.env_var_sources)?;
            }
            if !checksums.do_checksums_match(&lambda_fn.path.rel)? {
                let build_manifest = LambdaFnBuild::in_api_dir(lambda_fn.clone(), project.clone())
                    .build()
                    .await?;
                aws.sdk_clients
                    .lambda
                    .update_function_code()
                    .function_name(&lambda_fn.fn_name)
                    .zip_file(Blob::new(fs::read(&build_manifest.archive_path)?))
                    .send()
                    .await
                    .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?;
                checksums.update_all_checksums(&build_manifest.fn_sources)?;
                // todo wait for publish to finish
            }
            updating_fn.clone()
        }
    };

    match &components.route {
        None => {
            let integration =
                create_integration(&aws.sdk_clients, &aws.api.id, &synced_fn.arn).await?;
            aws.resources
                .created_gateway_integration(integration.clone())?;
            aws.resources.created_gateway_route(
                create_route(
                    &aws.sdk_clients,
                    &aws.api.id,
                    &lambda_fn.route_key,
                    &integration.id,
                )
                .await?,
            )?;
        }
        Some(route) => match &components.integration {
            None => {
                let integration =
                    create_integration(&aws.sdk_clients, &aws.api.id, &synced_fn.arn).await?;
                aws.resources
                    .created_gateway_integration(integration.clone())?;
                aws.resources.created_gateway_route(
                    update_route_target(&aws.sdk_clients, &aws.api.id, &route.id, &integration.id)
                        .await?,
                )?;
            }
            Some(integration) => {
                if integration.integration_uri.as_str() != synced_fn.arn.as_str() {
                    aws.resources.created_gateway_integration(
                        update_integration_uri(
                            &aws.sdk_clients,
                            &aws.api.id,
                            &integration.id,
                            &synced_fn.arn,
                        )
                        .await?,
                    )?;
                }
            }
        },
    };
    Ok(())
}
