use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, Error};
use aws_sdk_iam::types::Role;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::types::{FunctionCode, Runtime};

pub async fn create_fn(
    lambda: &aws_sdk_lambda::Client,
    function_name: &str,
    code_path: &PathBuf,
    role: &Role,
) -> Result<(), Error> {
    let start = std::time::Instant::now();
    loop {
        let result = lambda
            .create_function()
            .function_name(function_name)
            .runtime(Runtime::Nodejs20x)
            .role(&role.arn)
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
    println!(
        "debug: deployed in {}ms",
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
