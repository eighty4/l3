use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;

use anyhow::{anyhow, Error};
use aws_sdk_iam::types::Role;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::types::{FunctionCode, Runtime};

use crate::lambda::HttpMethod;

pub type FunctionArn = String;

pub async fn create_fn(
    lambda: &aws_sdk_lambda::Client,
    function_name: &str,
    code_path: &PathBuf,
    file_path: String,
    role: &Role,
) -> Result<FunctionArn, Error> {
    let handler = format!(
        "{}.handler",
        file_path.trim_end_matches(".js").trim_end_matches(".mjs")
    );
    let start = std::time::Instant::now();
    loop {
        let result = lambda
            .create_function()
            .function_name(function_name)
            .runtime(Runtime::Nodejs20x)
            .role(&role.arn)
            .handler(handler.clone())
            .code(
                FunctionCode::builder()
                    .zip_file(Blob::new(fs::read(code_path)?))
                    .build(),
            )
            .send()
            .await;
        match result {
            Ok(result) => {
                println!(
                    "debug: deployed in {}ms",
                    (std::time::Instant::now() - start).as_millis()
                );
                return Ok(result.function_arn.unwrap());
            }
            Err(err) => {
                if std::time::Instant::now() - start > Duration::from_secs(20) {
                    println!("timeout creating fn {function_name}");
                    exit(1);
                }
                if let Some(service_error) = err.as_service_error() {
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
}

pub async fn update_fn(
    lambda: &aws_sdk_lambda::Client,
    function_name: &str,
    code_path: &PathBuf,
) -> Result<FunctionArn, Error> {
    Ok(lambda
        .update_function_code()
        .function_name(function_name)
        .zip_file(Blob::new(fs::read(code_path)?))
        .send()
        .await?
        .function_arn
        .unwrap())
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

pub async fn add_apigateway_permission(
    lambda: &aws_sdk_lambda::Client,
    fn_arn: String,
    region: String,
    account_id: String,
    api_id: String,
    api_method: HttpMethod,
    api_path: String,
) -> Result<(), Error> {
    let source_arn = format!(
        "arn:aws:execute-api:{region}:{account_id}:{api_id}/development/{api_method}/{api_path}"
    );
    lambda
        .add_permission()
        .statement_id(format!("{api_id}_development"))
        .function_name(fn_arn)
        .action("lambda:InvokeFunction")
        .principal("apigateway.amazonaws.com")
        .source_arn(source_arn)
        .send()
        .await?;
    Ok(())
}
