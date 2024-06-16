use std::path::PathBuf;
use std::{env, fs};

use crate::aws::iam::create_lambda_role;
use crate::aws::lambda::{create_fn, does_fn_exist, update_fn};
use crate::aws::load_sdk_config;
use crate::code::create_archive;
use crate::config;
use crate::lambda::{HttpMethod, LambdaFn};

pub(crate) async fn sync_project() -> Result<(), anyhow::Error> {
    let project_name = match config::project_name()? {
        None => panic!("need a l3.yml file with a `project_name: ` string"),
        Some(project_name) => project_name,
    };
    println!("syncing project {project_name}");
    let sdk_config = load_sdk_config().await;
    println!(
        "aws sdk configured for region {}",
        sdk_config.region().unwrap()
    );
    let iam = aws_sdk_iam::Client::new(&sdk_config);
    let lambda = aws_sdk_lambda::Client::new(&sdk_config);

    let lambda_role = create_lambda_role(&iam, &project_name).await?;
    let lambda_fns = read_lambdas_from_current_dir(&project_name)?;

    if lambda_fns.is_empty() {
        println!("no lambdas found");
        return Ok(());
    }

    let code_path = create_archive()?;

    for lambda_fn in &lambda_fns {
        println!(
            "{} found in {}",
            lambda_fn.name,
            lambda_fn
                .path
                .strip_prefix(env::current_dir()?)?
                .to_string_lossy()
        );
        if does_fn_exist(&lambda, lambda_fn.name.as_str()).await? {
            update_fn(&lambda, lambda_fn.name.as_str(), &code_path).await?;
            println!("✔ updated");
        } else {
            create_fn(&lambda, lambda_fn.name.as_str(), &code_path, &lambda_role).await?;
            println!("✔ created");
        }
    }

    Ok(())
}

fn read_lambdas_from_current_dir(project_name: &String) -> Result<Vec<LambdaFn>, anyhow::Error> {
    read_lambdas_from_dir(project_name, env::current_dir()?)
}

fn read_lambdas_from_dir(
    project_name: &String,
    dir_path: PathBuf,
) -> Result<Vec<LambdaFn>, anyhow::Error> {
    let mut lambdas: Vec<LambdaFn> = Vec::new();
    for dir_entry_result in fs::read_dir(dir_path)? {
        let dir_entry = dir_entry_result?;
        let path = dir_entry.path();
        let ignored = ["target", "node_modules", ".git", ".github"].to_vec();
        if ignored.contains(&path.file_name().unwrap().to_string_lossy().as_ref()) {
            continue;
        } else if path.is_dir() {
            lambdas.append(&mut read_lambdas_from_dir(project_name, path)?)
        } else {
            let maybe_lambda = match path.file_name().unwrap().to_string_lossy().as_ref() {
                "delete.js" | "delete.mjs" => Some(HttpMethod::Delete),
                "get.js" | "get.mjs" => Some(HttpMethod::Get),
                "patch.js" | "patch.mjs" => Some(HttpMethod::Patch),
                "post.js" | "post.mjs" => Some(HttpMethod::Post),
                "put.js" | "put.mjs" => Some(HttpMethod::Put),
                _ => None,
            };
            if let Some(method) = maybe_lambda {
                lambdas.push(LambdaFn::new(project_name, method, path));
            }
        }
    }
    Ok(lambdas)
}
