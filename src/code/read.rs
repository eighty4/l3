use std::fs;
use std::path::{Path, PathBuf};

use crate::code::env::EnvVarSources;
use crate::code::parse::parse_module_for_lambda_handlers;
use crate::code::source::SourceFile;
use crate::lambda::{LambdaFn, RouteKey};

pub fn read_route_dir_for_lambdas(
    project_dir: &Path,
    project_name: &String,
) -> Result<Vec<LambdaFn>, anyhow::Error> {
    let mut lambdas = Vec::new();
    for lambda_path in recursively_read_route_dir(PathBuf::from("routes"))? {
        let lambda_fns = parse_module_for_lambda_handlers(&lambda_path)?;
        if lambda_fns.is_empty() {
            // todo warning if env files without any lambdas in a route dir
            continue;
        }
        let http_path = create_http_path_from_source_path(&lambda_path);
        let source_file = SourceFile::create(lambda_path, project_dir.to_path_buf())?;
        for (http_method, handler_fn) in lambda_fns {
            let route_key = RouteKey::new(http_method, http_path.clone());
            let env_var_sources = EnvVarSources::new(project_dir, &route_key)?;
            lambdas.push(LambdaFn::new(
                env_var_sources,
                handler_fn,
                project_name,
                route_key,
                source_file.clone(),
            ));
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

fn recursively_read_route_dir(dir_path: PathBuf) -> Result<Vec<PathBuf>, anyhow::Error> {
    let mut lambdas: Vec<PathBuf> = Vec::new();
    for path in recursively_read_dirs(&dir_path)? {
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        match file_name.as_ref() {
            "lambda.ts" | "lambda.js" | "lambda.mjs" => lambdas.push(path),
            _ => {
                // todo warning about unresolvable env and non route source files
            }
        }
    }
    Ok(lambdas)
}

pub fn recursively_read_dirs(dir_path: &PathBuf) -> Result<Vec<PathBuf>, anyhow::Error> {
    let mut files: Vec<PathBuf> = Vec::new();
    for dir_entry_result in fs::read_dir(dir_path)? {
        let dir_entry = dir_entry_result?;
        let path = dir_entry.path();
        if path.is_dir() {
            files.append(&mut recursively_read_dirs(&path)?);
        } else {
            files.push(path);
        }
    }
    Ok(files)
}
