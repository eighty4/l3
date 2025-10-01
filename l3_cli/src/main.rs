mod build;

use build::BuildCommand;
use clap::{Parser, Subcommand};
use std::process::exit;
use LLLCommand::*;

#[derive(Debug, thiserror::Error)]
enum LLLCommandRunError {
    #[error("current directory does not have a ./routes directory with Lambda functions")]
    LambdasNotFound,
}

type LLLCommandRunResult = Result<(), LLLCommandRunError>;

trait LLLCommandRun {
    async fn run(&self) -> LLLCommandRunResult;
}

#[derive(Parser)]
#[command(author, version, about)]
struct LLLCli {
    #[command(subcommand)]
    command: LLLCommand,
}

#[derive(Subcommand)]
enum LLLCommand {
    #[clap(about = "Build Lambda functions")]
    Build(BuildCommand),
}

#[tokio::main]
async fn main() {
    let result = match LLLCli::parse().command {
        Build(build) => build.run().await,
    };
    if let Err(err) = result {
        println!("\x1b[0;31;1merror:\x1b[0m {err}");
        exit(1);
    }
}
