use anyhow::anyhow;
use aws_sdk_apigatewayv2::types::ProtocolType;

pub async fn create_api(
    api_gateway: &aws_sdk_apigatewayv2::Client,
    project_name: &String,
    stage_name: &String,
) -> Result<String, anyhow::Error> {
    let create_api_output = api_gateway
        .create_api()
        .name(format!("l3-{project_name}-api"))
        .description(format!("L3 API for project {project_name}"))
        .protocol_type(ProtocolType::Http)
        .send()
        .await?;
    let api_id = create_api_output.api_id.unwrap();
    api_gateway
        .create_stage()
        .api_id(&api_id)
        .stage_name(stage_name)
        .auto_deploy(true)
        .send()
        .await?;
    Ok(api_id)
}

pub async fn does_api_exist(
    api_gateway: &aws_sdk_apigatewayv2::Client,
    api_id: &String,
) -> Result<bool, anyhow::Error> {
    match api_gateway.get_api().api_id(api_id).send().await {
        Ok(_) => Ok(true),
        Err(err) => {
            let service_error = err.as_service_error();
            if service_error.is_some() && service_error.unwrap().is_not_found_exception() {
                Ok(false)
            } else {
                Err(anyhow!("does_api_exist error from aws sdk {}", err))
            }
        }
    }
}
