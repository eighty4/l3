use crate::lambda::LambdaFn;
use crate::notification::{LambdaEventKind, LambdaUpdateResult};
use crate::project::Lx3Project;
use crate::task::launch::LaunchedTask;
use std::sync::Arc;

pub fn delete_fn(project: Arc<Lx3Project>, lambda_fn: Arc<LambdaFn>) -> LaunchedTask {
    Box::pin(delete_fn_inner(project, lambda_fn))
}

async fn delete_fn_inner(
    project: Arc<Lx3Project>,
    lambda_fn: Arc<LambdaFn>,
) -> Result<(), anyhow::Error> {
    project.send_lambda_event(lambda_fn.clone(), LambdaEventKind::Removing);
    let result = match delete_api_gateway_and_lambda_fn_resources(&project, lambda_fn.clone()).await
    {
        Ok(_) => LambdaUpdateResult::Success,
        Err(err) => LambdaUpdateResult::Failure(err.to_string()),
    };
    project.send_lambda_event(lambda_fn.clone(), LambdaEventKind::Removed(result));
    Ok(())
}

async fn delete_api_gateway_and_lambda_fn_resources(
    project: &Arc<Lx3Project>,
    lambda_fn: Arc<LambdaFn>,
) -> Result<(), anyhow::Error> {
    let aws = project.aws();
    let components = aws.resources.resources_for_fn(&lambda_fn).await?;
    if let Some(route) = components.route {
        aws.sdk_clients
            .api_gateway
            .delete_route()
            .api_id(&aws.api.id)
            .route_id(&route.id)
            .send()
            .await?;
        aws.resources.deleted_gateway_route(route)?;
    }
    if let Some(integration) = components.integration {
        aws.sdk_clients
            .api_gateway
            .delete_integration()
            .api_id(&aws.api.id)
            .integration_id(&integration.id)
            .send()
            .await?;
        aws.resources.deleted_gateway_integration(integration)?;
    }
    if let Some(removing_fn) = components.function {
        aws.sdk_clients
            .lambda
            .delete_function()
            .function_name(&removing_fn.name)
            .send()
            .await?;
        aws.resources.deleted_lambda_function(removing_fn)?;
    }
    Ok(())
}
