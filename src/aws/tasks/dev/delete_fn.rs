use crate::aws::state::DeployedProjectState;
use crate::lambda::LambdaFn;
use crate::notification::{LambdaEventKind, LambdaUpdateResult};
use crate::project::Lx3ProjectDeets;
use crate::task::launch::LaunchedTask;
use std::sync::Arc;

pub fn delete_fn(project_deets: Arc<Lx3ProjectDeets>, lambda_fn: Arc<LambdaFn>) -> LaunchedTask {
    Box::pin(delete_fn_inner(project_deets, lambda_fn))
}

async fn delete_fn_inner(
    project_deets: Arc<Lx3ProjectDeets>,
    lambda_fn: Arc<LambdaFn>,
) -> Result<(), anyhow::Error> {
    project_deets.send_lambda_event(lambda_fn.clone(), LambdaEventKind::Removing);
    let result =
        match delete_api_gateway_and_lambda_fn_resources(&project_deets, lambda_fn.clone()).await {
            Ok(_) => LambdaUpdateResult::Success,
            Err(err) => LambdaUpdateResult::Failure(err.to_string()),
        };
    project_deets.send_lambda_event(lambda_fn.clone(), LambdaEventKind::Removed(result));
    Ok(())
}

async fn delete_api_gateway_and_lambda_fn_resources(
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
    if let Some(route_id) = components.route {
        project_deets
            .aws
            .sdk_clients
            .api_gateway
            .delete_route()
            .api_id(&project_deets.aws.api.id)
            .route_id(route_id)
            .send()
            .await?;
    }
    if let Some((integration_id, _)) = components.integration {
        project_deets
            .aws
            .sdk_clients
            .api_gateway
            .delete_integration()
            .api_id(&project_deets.aws.api.id)
            .integration_id(integration_id)
            .send()
            .await?;
    }
    if let Some(fn_name) = components.function_name {
        project_deets
            .aws
            .sdk_clients
            .lambda
            .delete_function()
            .function_name(fn_name)
            .send()
            .await?;
    }
    Ok(())
}
