use std::{env, process};

use crate::aws::AwsApiConfig;
use crate::code::build::BuildMode;
use crate::dev::{develop_project, DevOptions};
use crate::init::{init_project, InitOptions};
use crate::sync::{sync_project, SyncOptions};
use clap::{Parser, Subcommand};

mod aws;
mod code;
mod config;
mod dev;
mod init;
mod lambda;
mod notification;
mod project;
mod sync;
mod task;
mod ui;

#[cfg(test)]
mod lambda_test;

#[cfg(test)]
mod testing;

#[derive(Parser)]
#[command(author, version, about)]
struct LambdaX3Cli {
    #[command(subcommand)]
    command: LambdaX3Command,
}

#[derive(Subcommand)]
enum LambdaX3Command {
    #[clap(about = "Create a project in the current directory")]
    Init(InitArgs),
    #[clap(about = "Deploy project resources to Lambda and API Gateway")]
    Sync(SyncArgs),
    #[clap(about = "Watch project directory and sync updates")]
    Dev(DevArgs),
}

#[derive(Parser, Debug)]
struct InitArgs {
    #[clap(
        long,
        value_name = "PROJECT_NAME",
        help = "Defaults to current directory name"
    )]
    project_name: Option<String>,
}

impl From<InitArgs> for InitOptions {
    fn from(args: InitArgs) -> Self {
        Self {
            project_name: args.project_name.unwrap_or_else(|| {
                env::current_dir()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            }),
        }
    }
}

#[derive(Parser, Debug)]
struct SyncArgs {
    #[clap(
        long,
        value_name = "API_ID",
        long_help = "Configure the project's API Gateway ID and cache in .l3"
    )]
    api_id: Option<String>,
    #[clap(
        long,
        default_value = "false",
        long_help = "Clear cache before sync and cleanly rebuild all Lambdas"
    )]
    clean: bool,
    #[clap(
        long,
        default_value = "false",
        long_help = "Auto approve AWS Region and API Gateway ID before syncing"
    )]
    confirm: bool,
    #[clap(
        long,
        default_value = "false",
        long_help = "Create a release build of Lambda Function artifacts"
    )]
    release: bool,
}

impl TryFrom<SyncArgs> for SyncOptions {
    type Error = anyhow::Error;

    fn try_from(args: SyncArgs) -> Result<Self, Self::Error> {
        Ok(Self {
            aws: AwsApiConfig {
                api_id: args.api_id,
                stage_name: None,
            },
            auto_confirm: args.confirm,
            build_mode: match args.release {
                true => BuildMode::Release,
                false => BuildMode::Debug,
            },
            clear_cache: args.clean,
            project_dir: env::current_dir()?,
            project_name: match config::project_name()? {
                None => panic!("need a l3.yml file with a `project_name: ` string"),
                Some(project_name) => project_name,
            },
        })
    }
}

#[derive(Parser, Debug)]
struct DevArgs {
    #[clap(
        long,
        value_name = "API_ID",
        long_help = "Configure the project's API Gateway ID and cache in .l3"
    )]
    api_id: Option<String>,
    #[clap(
        long,
        default_value = "false",
        long_help = "Clear cache before sync and cleanly rebuild all Lambdas"
    )]
    clean: bool,
    #[clap(
        long,
        default_value = "false",
        long_help = "Auto approve AWS Region and API Gateway ID before syncing"
    )]
    confirm: bool,
}

impl TryFrom<DevArgs> for DevOptions {
    type Error = anyhow::Error;

    fn try_from(args: DevArgs) -> Result<Self, Self::Error> {
        Ok(Self {
            aws: AwsApiConfig {
                api_id: args.api_id,
                stage_name: None,
            },
            auto_confirm: args.confirm,
            clear_cache: args.clean,
            project_dir: env::current_dir()?,
            project_name: match config::project_name()? {
                None => panic!("need a l3.yml file with a `project_name: ` string"),
                Some(project_name) => project_name,
            },
        })
    }
}

#[tokio::main]
async fn main() {
    if let Err(err) = exec_cmd(LambdaX3Cli::parse().command).await {
        println!("error: {err}");
        process::exit(1);
    }
}

async fn exec_cmd(cmd: LambdaX3Command) -> Result<(), anyhow::Error> {
    match cmd {
        LambdaX3Command::Init(args) => init_project(InitOptions::from(args)),
        LambdaX3Command::Sync(args) => sync_project(SyncOptions::try_from(args)?).await,
        LambdaX3Command::Dev(args) => develop_project(DevOptions::try_from(args)?).await,
    }
}
