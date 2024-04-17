use std::time::Duration;
use std::{fs, io, path::PathBuf};

use anyhow::{anyhow, Error};
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::types::{FunctionCode, Runtime};
use base64::Engine;
use sha2::{Digest, Sha256};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let function_name = std::env::args().nth(1).expect("fn name required");

    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region(region_provider)
        .load()
        .await;
    let iam = aws_sdk_iam::Client::new(&config);
    let lambda = aws_sdk_lambda::Client::new(&config);

    deploy_fn(&iam, &lambda, function_name.as_str()).await?;

    Ok(())
}

async fn get_account_id(iam: &aws_sdk_iam::Client) -> Result<String, Error> {
    let user_arn = iam.get_user().send().await.unwrap().user.unwrap().arn;
    Ok(user_arn.split(':').nth(4).unwrap().to_string())
}

async fn deploy_fn(
    iam: &aws_sdk_iam::Client,
    lambda: &aws_sdk_lambda::Client,
    function_name: &str,
) -> Result<(), Error> {
    if does_fn_exist(lambda, function_name).await? {
        // update_fn();
    } else {
        create_fn(iam, lambda, function_name).await?;
    }
    Ok(())
}

async fn create_fn(
    iam: &aws_sdk_iam::Client,
    lambda: &aws_sdk_lambda::Client,
    function_name: &str,
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
                    .zip_file(Blob::new(fs::read("code/code.zip")?))
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
        "deployed {}",
        (std::time::Instant::now() - start).as_millis()
    );

    Ok(())
}

async fn does_fn_exist(
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

#[allow(dead_code)]
fn sha256_checksum(path: &PathBuf) -> Result<String, Error> {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(path)?;
    io::copy(&mut file, &mut hasher)?;
    let hash_bytes = hasher.finalize();
    Ok(base64::engine::general_purpose::STANDARD.encode(hash_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use io::Write;
    use temp_dir::TempDir;

    #[test]
    fn test() {
        let d = TempDir::new().expect("make temp dir");
        let p = d.path().join("file");
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&p)
            .expect("create file");
        f.write_all("content".as_bytes())
            .expect("write bytes to file");

        let result = sha256_checksum(&p);
        assert!(result.is_ok());
        assert_eq!(
            "7XACtDnprIRfIjV9giusFERzD722AW0+yUMil7nsn3M=",
            result.unwrap()
        );
    }
}
