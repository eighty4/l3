use crate::code::build::{BuildMode, LambdaFnBuild};
use crate::code::runtime::RuntimeConfig;
use crate::code::source::path::{FunctionBuildDir, SourcePath};
use crate::lambda::{LambdaFn, RouteKey};
use crate::project::Lx3Project;
use anyhow::anyhow;
use std::path::PathBuf;

pub struct BuildFunctionOptions {
    pub build_mode: BuildMode,
    pub p: PathBuf,
    pub project_dir: PathBuf,
    pub project_name: String,
}

pub async fn build_function(opts: BuildFunctionOptions) -> Result<(), anyhow::Error> {
    debug_assert!(opts.p.is_relative());
    if !opts.p.is_file() {
        return Err(anyhow!("path {} does not exist", opts.p.to_string_lossy()));
    }
    let is_routes_source_file: bool = {
        let mut yes = false;
        for c in opts.p.components() {
            if c.as_os_str().to_string_lossy() == "routes" {
                yes = true;
                break;
            }
        }
        yes
    };
    if !is_routes_source_file {
        return Err(anyhow!(
            "path {} must be a ./routes directory path",
            opts.p.to_string_lossy()
        ));
    }

    let (runtime_config, _) = RuntimeConfig::new(opts.project_dir.clone());
    let (project, _) = Lx3Project::builder()
        .build_mode(opts.build_mode.clone())
        .runtime_config(runtime_config)
        .build(opts.project_dir.clone(), opts.project_name.clone());

    let source_path = SourcePath::from_rel(&project.dir, opts.p.clone());
    let route_key = RouteKey::from_route_key_string("GET /foo".to_string())?;
    let lambda_fn = LambdaFn::new("".to_string(), source_path, project.clone(), route_key);
    let build_dir = FunctionBuildDir::Unsynced(project.clone(), lambda_fn.clone());
    let _build_manifest = LambdaFnBuild::new(build_dir, lambda_fn, project.clone())
        .build()
        .await?;

    Ok(())
}
