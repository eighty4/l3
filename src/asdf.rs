use std::{fs, io, path::PathBuf};
use std::fs::File;

use anyhow::Error;
use aws_config::meta::region::RegionProviderChain;
use base64::Engine;
use sha2::{Digest, Sha256};
use toml::Table;
use zip::write::ZipWriter;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let l3 = L3Cli::parse();
    match l3.command {
        L3Cmd::Fn(cmd) => {
            fn_dev_sync(cmd.function_name, cmd.src_dir).await?;
        }
    }
    // match fs::read("l3.toml") {
    //
    // }
    let value = "".parse::<Table>().unwrap();

    // let resp = lambda.list_functions().send().await?;
    // println!("Functions:");
    // let functions = resp.functions().unwrap_or_default();
    // for function in functions {
    //     println!("  {}", function.function_name().unwrap_or_default());
    // }
    // println!("Found {} functions", functions.len());

    Ok(())
}

async fn fn_dev_sync(fn_name: String, src_dir: Option<PathBuf>) -> Result<(), Error> {
    zip_src_dir(&PathBuf::from("github-oauth-redirect"))?;
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let lambda = aws_sdk_lambda::Client::new(&config);
    let result = lambda.get_function().function_name(fn_name).send().await?;
    let configuration = result.configuration().expect("fn config");
    let checksum = configuration
        .code_sha256().expect("fn checksum");
    println!("{}", checksum);
    Ok(())
}
use std::io::Write;
use zip::write::FileOptions;

fn zip_src_dir(src_dir: &PathBuf) -> Result<(), Error> {
    for f in fs::read_dir(src_dir)? {

    }
    let file = File::create("foo.zip")?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.add_directory("github-oauth-redirect", options)?;
    zip.finish()?;
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
    use io::Write;

    use temp_dir::TempDir;

    use super::*;

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
