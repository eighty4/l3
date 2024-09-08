use crate::code::build::LambdaFnBuild;
use crate::lambda::LambdaFn;
use crate::notification::{
    LambdaEvent, LambdaEventKind, LambdaNotification, LambdaUpdateKind, LambdaUpdateResult,
};
use crate::project::Lx3ProjectDeets;
use crate::task::launch::LaunchedTask;
use anyhow::anyhow;
use aws_sdk_lambda::primitives::Blob;
use std::fs;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub fn update_code(
    notification_tx: UnboundedSender<LambdaNotification>,
    project_deets: Arc<Lx3ProjectDeets>,
    lambda_fn: Arc<LambdaFn>,
) -> LaunchedTask {
    Box::pin(update_code_inner(notification_tx, project_deets, lambda_fn))
}

async fn update_code_inner(
    notification_tx: UnboundedSender<LambdaNotification>,
    project_deets: Arc<Lx3ProjectDeets>,
    lambda_fn: Arc<LambdaFn>,
) -> Result<(), anyhow::Error> {
    notification_tx.send(LambdaNotification::Lambda(LambdaEvent {
        lambda_fn: lambda_fn.clone(),
        kind: LambdaEventKind::Updating(LambdaUpdateKind::Code),
    }))?;
    let result = match build_and_update(project_deets, lambda_fn.clone()).await {
        Ok(_) => LambdaUpdateResult::Success,
        Err(err) => LambdaUpdateResult::Failure(err.to_string()),
    };
    notification_tx.send(LambdaNotification::Lambda(LambdaEvent {
        lambda_fn,
        kind: LambdaEventKind::Updated(LambdaUpdateKind::Code, result),
    }))?;
    Ok(())
}

async fn build_and_update(
    project_deets: Arc<Lx3ProjectDeets>,
    lambda_fn: Arc<LambdaFn>,
) -> Result<(), anyhow::Error> {
    let archive = LambdaFnBuild::new(lambda_fn.clone(), project_deets.clone())
        .create_code_archive()
        .await?;
    project_deets
        .aws
        .sdk_clients
        .lambda
        .update_function_code()
        .function_name(&lambda_fn.fn_name)
        .zip_file(Blob::new(fs::read(&archive.path)?))
        .send()
        .await
        .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?;
    // todo checksums.update_checksum(params.lambda_fn.path.rel.clone())?;
    // todo wait for publish to finish
    Ok(())
}
