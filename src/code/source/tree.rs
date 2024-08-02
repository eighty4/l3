use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::task::JoinSet;

use crate::code::checksum::ChecksumTree;
use crate::code::env::EnvVarSources;
use crate::code::parse::parse_source_file;
use crate::code::read::parallel;
use crate::code::source::path::SourcePath;
use crate::code::source::SourceFile;
use crate::lambda::{LambdaFn, RouteKey};
use crate::project::Lx3ProjectDeets;

pub struct SourceTree {
    checksums: HashMap<RouteKey, ChecksumTree>,
    pub lambdas: HashMap<RouteKey, Arc<LambdaFn>>,
    project_details: Arc<Lx3ProjectDeets>,
    pub sources: HashMap<PathBuf, Arc<SourceFile>>,
}

impl SourceTree {
    pub fn new(project_details: Arc<Lx3ProjectDeets>) -> Self {
        Self {
            checksums: HashMap::new(),
            lambdas: HashMap::new(),
            project_details,
            sources: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<(), anyhow::Error> {
        let mut parse_route_sources: JoinSet<Result<SourceFile, anyhow::Error>> = JoinSet::new();
        for path in
            parallel::recursively_read_dirs(&self.project_details.project_dir.join("routes"))
                .await?
        {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            match file_name.as_ref() {
                "lambda.js" | "lambda.mjs" | "lambda.py" | "lambda.ts" => {}
                _ => {
                    // todo warning about unresolvable env and non route source files
                    continue;
                }
            }
            let project_deets = self.project_details.clone();
            parse_route_sources.spawn(async move {
                let source_path = SourcePath::from_abs(&project_deets.project_dir, path);
                parse_source_file(source_path.clone(), &project_deets.sources)
            });
        }
        let mut parse_lambda_fns: JoinSet<Result<Vec<LambdaFn>, anyhow::Error>> = JoinSet::new();
        while let Some(parse_result) = parse_route_sources.join_next().await {
            let source_file = Arc::new(parse_result??);
            self.sources
                .insert(source_file.path.rel.clone(), source_file.clone());
            let project_deets = self.project_details.clone();
            parse_lambda_fns.spawn(async move {
                let mut lambdas = Vec::new();
                let handler_fns = source_file.collect_handler_fn_names()?;
                if handler_fns.is_empty() {
                    // todo warning if env files without any lambdas in a route dir
                } else {
                    let http_path = RouteKey::extract_http_path(&source_file.path.rel).unwrap();
                    // todo move to SourceFile
                    for (http_method, handler_fn) in handler_fns {
                        let route_key = RouteKey::new(http_method, http_path.clone());
                        let env_var_sources =
                            EnvVarSources::new(&project_deets.project_dir, &route_key)?;
                        lambdas.push(LambdaFn::new(
                            env_var_sources,
                            handler_fn,
                            source_file.path.clone(),
                            project_deets.clone(),
                            route_key,
                        ));
                    }
                }
                Ok(lambdas)
            });
        }
        while let Some(parse_result) = parse_lambda_fns.join_next().await {
            for lambda_fn in parse_result?? {
                self.lambdas
                    .insert(lambda_fn.route_key.clone(), Arc::new(lambda_fn));
            }
        }
        Ok(())
    }
}
