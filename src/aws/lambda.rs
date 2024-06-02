use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, Error};
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::types::{FunctionCode, FunctionUrlAuthType, Runtime};

use crate::aws::iam::get_account_id;

pub async fn create_fn(
    iam: &aws_sdk_iam::Client,
    lambda: &aws_sdk_lambda::Client,
    function_name: &str,
    code_path: &PathBuf,
) -> Result<(), Error> {
    let account_id = get_account_id(iam).await?;
    let policy_result = iam
        .create_policy()
        .policy_name(format!("l3-policy-{}", function_name))
        .policy_document(
            include_str!("l3-policy.json")
                .replace("$$$ACCOUNT_ID$$$", account_id.as_str())
                .replace("$$$FUNCTION_NAME$$$", function_name),
        )
        .send()
        .await?;

    let role_name = format!("l3-role-{}", function_name);
    let role_result = iam
        .create_role()
        .role_name(role_name.clone())
        .assume_role_policy_document(include_str!("l3-trust.json"))
        .send()
        .await?;

    iam.attach_role_policy()
        .role_name(role_result.role.unwrap().role_name)
        .policy_arn(policy_result.policy.unwrap().arn.unwrap())
        .send()
        .await?;

    let start = std::time::Instant::now();
    tokio::time::sleep(Duration::from_secs(5)).await;
    loop {
        let result = lambda
            .create_function()
            .function_name(function_name)
            .runtime(Runtime::Nodejs20x)
            .role(format!(
                "arn:aws:iam::{}:role/l3-role-{}",
                account_id, function_name
            ))
            .handler("handler")
            .code(
                FunctionCode::builder()
                    .zip_file(Blob::new(fs::read(code_path)?))
                    .build(),
            )
            .send()
            .await;
        match result {
            Ok(_) => break,
            Err(err) => {
                if std::time::Instant::now() - start > Duration::from_secs(20) {
                    println!("wtf");
                }
                let maybe_service_error = err.as_service_error();
                if maybe_service_error.is_some() {
                    let service_error = maybe_service_error.unwrap();
                    if service_error.is_invalid_parameter_value_exception()
                        && service_error.to_string().contains("assumed")
                    {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                }
            }
        }
    }
    println!(
        "deployed in {}ms",
        (std::time::Instant::now() - start).as_millis()
    );

    Ok(())
}

pub async fn update_fn(
    lambda: &aws_sdk_lambda::Client,
    function_name: &str,
    code_path: &PathBuf,
) -> Result<(), Error> {
    lambda
        .update_function_code()
        .function_name(function_name)
        .zip_file(Blob::new(fs::read(code_path)?))
        .send()
        .await?;
    Ok(())
}

#[allow(dead_code)]
async fn create_fn_uri(
    lambda: &aws_sdk_lambda::Client,
    function_name: &str,
) -> Result<String, Error> {
    let result = lambda
        .create_function_url_config()
        .function_name(function_name)
        .auth_type(FunctionUrlAuthType::None)
        .send()
        .await?;
    Ok(result.function_url)
}

pub async fn does_fn_exist(
    lambda: &aws_sdk_lambda::Client,
    function_name: &str,
) -> Result<bool, Error> {
    match lambda
        .get_function()
        .function_name(function_name)
        .send()
        .await
    {
        Ok(_) => Ok(true),
        Err(err) => {
            let service_error = err.as_service_error();
            if service_error.is_some() && service_error.unwrap().is_resource_not_found_exception() {
                Ok(false)
            } else {
                Err(anyhow!("does_fn_exist error {}", err))
            }
        }
    }
}
