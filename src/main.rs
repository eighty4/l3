use std::{fs, io, path::PathBuf};

use anyhow::Error;
use aws_config::meta::region::RegionProviderChain;
use base64::Engine;
use sha2::{Digest, Sha256};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let lambda = aws_sdk_lambda::Client::new(&config);

    let resp = lambda.list_functions().send().await?;
    println!("Functions:");
    let functions = resp.functions().unwrap_or_default();
    for function in functions {
        println!("  {}", function.function_name().unwrap_or_default());
    }
    println!("Found {} functions", functions.len());

    Ok(())
}
