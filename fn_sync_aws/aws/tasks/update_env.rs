use crate::aws::tasks::api::lambda::wait_for_fn_update_successful;
use crate::lambda::LambdaFn;
use crate::notification::{LambdaEventKind, LambdaUpdateKind, LambdaUpdateResult};
use crate::project::Lx3Project;
use crate::task::launch::LaunchedTask;
use anyhow::anyhow;
use aws_sdk_lambda::types::Environment;
use std::sync::Arc;

pub fn update_env(project: Arc<Lx3Project>, lambda_fn: Arc<LambdaFn>) -> LaunchedTask {
    Box::pin(update_env_inner(project, lambda_fn))
}

async fn update_env_inner(
    project: Arc<Lx3Project>,
    lambda_fn: Arc<LambdaFn>,
) -> Result<(), anyhow::Error> {
    project.send_lambda_event(
        lambda_fn.clone(),
        LambdaEventKind::Updating(LambdaUpdateKind::Env),
    );
    let result = match update_fn_config_env(&project, lambda_fn.clone()).await {
        Ok(_) => LambdaUpdateResult::Success,
        Err(err) => LambdaUpdateResult::Failure(err.to_string()),
    };
    project.send_lambda_event(
        lambda_fn.clone(),
        LambdaEventKind::Updated(LambdaUpdateKind::Code, result),
    );
    Ok(())
}

async fn update_fn_config_env(
    project: &Arc<Lx3Project>,
    lambda_fn: Arc<LambdaFn>,
) -> Result<(), anyhow::Error> {
    let aws = project.aws();
    let environment = Environment::builder()
        .set_variables(lambda_fn.env_var_sources.read_env_variables()?)
        .build();
    aws.sdk_clients
        .lambda
        .update_function_configuration()
        .function_name(&lambda_fn.fn_name)
        .environment(environment)
        .send()
        .await
        .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?;
    wait_for_fn_update_successful(&aws.sdk_clients.lambda, &lambda_fn.fn_name).await?;
    // todo checksums.update_env_var_checksums(&params.lambda_fn.env_var_sources)?;
    Ok(())
}
