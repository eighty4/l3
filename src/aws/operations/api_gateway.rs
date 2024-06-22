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
