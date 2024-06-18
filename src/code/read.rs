use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::code::parse::parse_module_for_lambda_handlers;
use crate::lambda::{LambdaFn, RouteKey, SourceFile};

pub fn read_route_dir_for_lambdas(
    project_name: &String,
) -> Result<HashMap<RouteKey, LambdaFn>, anyhow::Error> {
    let mut lambdas = HashMap::new();
    for route_dir in recursively_read_route_dir(PathBuf::from("routes"))? {
        for source_path in route_dir.lambda_files {
            let lambda_fns = parse_module_for_lambda_handlers(&source_path)?;
            if lambda_fns.is_empty() {
                continue;
            }
            let http_path = create_http_path_from_source_path(&source_path);
            let source_file = SourceFile::try_from(source_path)?;
            for (http_method, handler_fn) in lambda_fns {
                let route_key = RouteKey::new(http_method, http_path.clone());
                lambdas.insert(
                    route_key.clone(),
                    LambdaFn::new(handler_fn, project_name, route_key, source_file.clone()),
                );
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
    #[allow(unused)]
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
                    if file_name.contains(".env") {
                        env_files.push(path)
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
