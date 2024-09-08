use crate::aws::clients::AwsClients;
use crate::aws::state::DeployedLambdaComponents;

pub struct RemoveFnParams {
    pub api_id: String,
    pub components: DeployedLambdaComponents,
}

/// Clean up task for `l3 sync` to remove any AWS components related to no longer needed functions.
/// Could be called to remove a fully deployed and integrated Lambda or lingering individual
/// components.
pub async fn perform_remove_fn(
    sdk_clients: &AwsClients,
    params: &RemoveFnParams,
) -> Result<(), anyhow::Error> {
    if let Some(route_id) = &params.components.route {
        sdk_clients
            .api_gateway
            .delete_route()
            .api_id(&params.api_id)
            .route_id(route_id)
            .send()
            .await?;
        println!("  ✔ Removed API Gateway Route {route_id}");
    }
    if let Some((integration_id, _)) = &params.components.integration {
        sdk_clients
            .api_gateway
            .delete_integration()
            .api_id(&params.api_id)
            .integration_id(integration_id)
            .send()
            .await?;
        println!("  ✔ Removed API Gateway Integration {integration_id}");
    }
    if let Some(fn_name) = &params.components.function_name {
        sdk_clients
            .lambda
            .delete_function()
            .function_name(fn_name)
            .send()
            .await?;
        println!("  ✔ Removed Lambda Function {fn_name}");
    }
    Ok(())
}
