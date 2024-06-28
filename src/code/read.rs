use std::fs;
use std::path::{Path, PathBuf};

use crate::code::env::{is_valid_env_file_name, EnvFile, EnvVarSources};
use crate::code::parse::parse_module_for_lambda_handlers;
use crate::code::source::SourceFile;
use crate::lambda::{LambdaFn, RouteKey};

pub fn read_route_dir_for_lambdas(
    project_dir: &Path,
    project_name: &String,
) -> Result<Vec<LambdaFn>, anyhow::Error> {
    let mut lambdas = Vec::new();
    for route_dir in recursively_read_route_dir(PathBuf::from("routes"))? {
        for source_path in route_dir.lambda_files {
            let lambda_fns = parse_module_for_lambda_handlers(&source_path)?;
            if lambda_fns.is_empty() {
                // todo warning if env files without any lambdas in a route dir
                continue;
            }
            let mut route_env_files = Vec::new();
            for env_file_path in &route_dir.env_files {
                route_env_files.push(EnvFile::create(
                    env_file_path.clone(),
                    project_dir.to_path_buf(),
                )?);
            }
            let http_path = create_http_path_from_source_path(&source_path);
            let source_file = SourceFile::create(source_path, project_dir.to_path_buf())?;
            for (http_method, handler_fn) in lambda_fns {
                let mut lambda_env_files = Vec::new();
                for env_file in &route_env_files {
                    if env_file.is_for_http_method(&http_method) {
                        lambda_env_files.push(env_file.clone())
                    }
                }
                let route_key = RouteKey::new(http_method, http_path.clone());
                let env_var_sources = EnvVarSources::new(lambda_env_files, route_key.clone())?;
                lambdas.push(LambdaFn::new(
                    env_var_sources,
                    handler_fn,
                    project_name,
                    route_key,
                    source_file.clone(),
                ));
            }
        }
    }
    Ok(lambdas)
}

// todo fix this bad code
fn create_http_path_from_source_path(source_path: &Path) -> String {
    let mut parts = Vec::new();
    for p in source_path.parent().unwrap().components().rev() {
        if p.as_os_str().to_string_lossy().as_ref() == "routes" {
            break;
        } else {
            parts.insert(0, p);
        }
    }
    PathBuf::from_iter(parts).to_string_lossy().to_string()
}

struct RouteDir {
    #[allow(unused)]
    dir_path: PathBuf,
    env_files: Vec<PathBuf>,
    lambda_files: Vec<PathBuf>,
}

fn recursively_read_route_dir(dir_path: PathBuf) -> Result<Vec<RouteDir>, anyhow::Error> {
    let mut route_dirs: Vec<RouteDir> = Vec::new();
    let mut env_files: Vec<PathBuf> = Vec::new();
    let mut lambda_files: Vec<PathBuf> = Vec::new();
    for dir_entry_result in fs::read_dir(&dir_path)? {
        let dir_entry = dir_entry_result?;
        let path = dir_entry.path();
        if path.is_dir() {
            route_dirs.append(&mut recursively_read_route_dir(path)?);
        } else {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            match file_name.as_ref() {
                "lambda.ts" | "lambda.js" | "lambda.mjs" => lambda_files.push(path),
                _ => {
                    if file_name.ends_with(".env") {
                        if is_valid_env_file_name(&file_name) {
                            env_files.push(path)
                        } else {
                            // todo warning about unresolvable env file
                        }
                    }
                }
            }
        }
    }
    route_dirs.push(RouteDir {
        dir_path,
        lambda_files,
        env_files,
    });
    Ok(route_dirs)
}
