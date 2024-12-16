use crate::code::build::LambdaFnBuild;
use crate::code::checksum::ChecksumTree;
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
    let aws = project.aws();
    let mut checksums =
        ChecksumTree::new(project.dir.clone(), &aws.api.id, &lambda_fn.fn_name).await?;
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
    Ok(())
}
