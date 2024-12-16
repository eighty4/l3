use anyhow::anyhow;
use aws_sdk_iam::types::Role;

pub(crate) async fn create_lambda_role(
    iam: &aws_sdk_iam::Client,
    project_name: &String,
) -> Result<Role, anyhow::Error> {
    let role_name = format!("l3-{project_name}-lambda-role");
    let mut role = get_role(iam, &role_name).await?;
    if role.is_none() {
        role = iam
            .create_role()
            .role_name(&role_name)
            .assume_role_policy_document(include_str!("l3-trust.json"))
            .send()
            .await?
            .role;
    }
    // todo check if attach-role_policy is idempotent?
    iam.attach_role_policy()
        .role_name(&role_name)
        .policy_arn("arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole")
        .send()
        .await?;
    Ok(role.unwrap())
}

async fn get_role(
    iam: &aws_sdk_iam::Client,
    role_name: &String,
) -> Result<Option<Role>, anyhow::Error> {
    match iam.get_role().role_name(role_name).send().await {
        Ok(get_role_output) => Ok(get_role_output.role),
        Err(err) => match err.as_service_error() {
            None => Err(anyhow!(err)),
            Some(service_error) => {
                if service_error.is_no_such_entity_exception() {
                    Ok(None)
                } else {
                    Err(anyhow!(err))
                }
            }
        },
    }
}
