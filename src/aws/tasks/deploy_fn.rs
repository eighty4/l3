use std::path::PathBuf;

use aws_sdk_apigatewayv2::types::IntegrationType;

use crate::aws::clients::AwsClients;
use crate::aws::operations::lambda::{create_fn, update_fn};
use crate::aws::tasks::DeployFnParams;
use crate::code::archive::CODE_ARCHIVE_PATH;

pub async fn perform_deploy_fn(
    sdk_clients: &AwsClients,
    params: DeployFnParams,
) -> Result<(), anyhow::Error> {
    let fn_arn = match params.deployed_components.function {
        None => {
            create_fn(
                &sdk_clients.lambda,
                &params.lambda_fn.fn_name,
                &PathBuf::from(CODE_ARCHIVE_PATH),
                &params.lambda_fn.handler_path(),
                &params.lambda_role_arn,
            )
            .await?
        }
        Some(_) => {
            update_fn(
                &sdk_clients.lambda,
                &params.lambda_fn.fn_name,
                &PathBuf::from(CODE_ARCHIVE_PATH),
            )
            .await?
        }
    };
    println!(
        "✔ {} {}",
        params
            .deployed_components
            .function
            .map_or("updated", |_| "created"),
        params.lambda_fn.fn_name
    );

    let integration_id = match params.deployed_components.integration {
        None => sdk_clients
            .api_gateway
            .create_integration()
            .api_id(&params.api_id)
            .integration_type(IntegrationType::AwsProxy)
            .integration_uri(&fn_arn)
            .payload_format_version("2.0")
            .send()
            .await?
            .integration_id
            .unwrap(),
        Some(id) => id,
    };

    match params.deployed_components.route {
        None => {
            sdk_clients
                .api_gateway
                .create_route()
                .api_id(&params.api_id)
                .route_key(&params.lambda_fn.route_key.to_route_key_string())
                .target(format!("integrations/{integration_id}"))
                .send()
                .await?;
        }
        Some(_) => {}
    };

    let source_arn = format!(
        "arn:aws:execute-api:{}:{}:{}/{}/{}/{}",
        params.region,
        params.account_id,
        params.api_id,
        params.stage_name,
        params.lambda_fn.route_key.http_method,
        params.lambda_fn.route_key.http_path
    );
    sdk_clients
        .lambda
        .add_permission()
        .statement_id(format!("{}_{}", params.api_id, params.stage_name))
        .function_name(&fn_arn)
        .action("lambda:InvokeFunction")
        .principal("apigateway.amazonaws.com")
        .source_arn(source_arn)
        .send()
        .await?;

    Ok(())
}
