use std::{fs, io, path::PathBuf, process};

use anyhow::Error;
use base64::Engine;
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};

use crate::init::init_project;
use crate::sync::sync_project;

mod aws;
mod code;
mod config;
mod init;
mod lambda;
mod sync;

#[derive(Parser)]
#[command(author, version, about)]
struct LambdaX3Cli {
    #[command(subcommand)]
    command: LambdaX3Command,
}

#[derive(Subcommand)]
enum LambdaX3Command {
    Init,
    Sync,
}

#[tokio::main]
async fn main() {
    if let Err(err) = exec_cmd(LambdaX3Cli::parse().command).await {
        println!("error: {err}");
        process::exit(1);
    }
}

async fn exec_cmd(cmd: LambdaX3Command) -> Result<(), Error> {
    match cmd {
        LambdaX3Command::Init => init_project(),
        LambdaX3Command::Sync => sync_project().await,
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
