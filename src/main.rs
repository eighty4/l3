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
