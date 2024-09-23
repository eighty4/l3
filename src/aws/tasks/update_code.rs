use crate::code::build::LambdaFnBuild;
use crate::lambda::LambdaFn;
use crate::notification::{LambdaEventKind, LambdaUpdateKind, LambdaUpdateResult};
use crate::project::Lx3Project;
use crate::task::launch::LaunchedTask;
use anyhow::anyhow;
use aws_sdk_lambda::primitives::Blob;
use std::fs;
use std::sync::Arc;

pub fn update_code(project: Arc<Lx3Project>, lambda_fn: Arc<LambdaFn>) -> LaunchedTask {
    Box::pin(update_code_inner(project, lambda_fn))
}

async fn update_code_inner(
    project: Arc<Lx3Project>,
    lambda_fn: Arc<LambdaFn>,
) -> Result<(), anyhow::Error> {
    project.send_lambda_event(
        lambda_fn.clone(),
        LambdaEventKind::Updating(LambdaUpdateKind::Code),
    );
    let result = match build_and_update_fn_code(&project, lambda_fn.clone()).await {
        Ok(_) => LambdaUpdateResult::Success,
        Err(err) => LambdaUpdateResult::Failure(err.to_string()),
    };
    project.send_lambda_event(
        lambda_fn.clone(),
        LambdaEventKind::Updated(LambdaUpdateKind::Code, result),
    );
    Ok(())
}

async fn build_and_update_fn_code(
    project: &Arc<Lx3Project>,
    lambda_fn: Arc<LambdaFn>,
) -> Result<(), anyhow::Error> {
    let archive = LambdaFnBuild::new(lambda_fn.clone(), project.clone())
        .create_code_archive()
        .await?;
    project
        .aws
        .sdk_clients
        .lambda
        .update_function_code()
        .function_name(&lambda_fn.fn_name)
        .zip_file(Blob::new(fs::read(&archive.path)?))
        .send()
        .await
        .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?;
    // todo update_checksum for each source file included in build
    // todo checksums.update_checksum(params.lambda_fn.path.rel.clone())?;
    // todo wait for publish to finish
    Ok(())
}
