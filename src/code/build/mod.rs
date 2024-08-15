use std::path::PathBuf;
use std::sync::Arc;

use crate::code::build::swc::SwcBuilder;
use crate::code::parse::parse_source_file;
use crate::code::source::path::{FunctionBuildDir, SourcePath};
use crate::code::source::{Language, SourceFile};
use crate::lambda::LambdaFn;
use crate::project::Lx3ProjectDeets;
use archiver::Archiver;

mod archiver;
mod swc;

#[cfg(test)]
mod archiver_test;
#[cfg(test)]
mod swc_test;

trait Builder {
    fn build(
        &self,
        source_file: &SourceFile,
        build_dir: &FunctionBuildDir,
    ) -> Result<SourcePath, anyhow::Error>;
}

#[derive(Clone)]
pub enum BuildMode {
    Debug,
    Release,
}

impl BuildMode {
    pub fn should_minify(&self) -> bool {
        match self {
            BuildMode::Debug => false,
            BuildMode::Release => true,
        }
    }
}

pub struct LambdaFnBuild {
    build_dir: FunctionBuildDir,
    // todo arc
    builder: Box<dyn Builder + Send + Sync>,
    entrypoint: SourcePath,
    project_details: Arc<Lx3ProjectDeets>,
}

impl LambdaFnBuild {
    pub fn new(lambda_fn: Arc<LambdaFn>, project_details: Arc<Lx3ProjectDeets>) -> Self {
        Self {
            build_dir: lambda_fn.build_dir(&project_details),
            builder: Box::new(match &lambda_fn.language {
                Language::JavaScript | Language::TypeScript => {
                    SwcBuilder::new(project_details.clone())
                }
                Language::Python => panic!(),
            }),
            entrypoint: lambda_fn.path.clone(),
            project_details,
        }
    }

    pub async fn build(&self) -> Result<Vec<SourcePath>, anyhow::Error> {
        let source_file = parse_source_file(
            self.entrypoint.clone(),
            self.project_details.runtime_config.clone(),
        )?;
        self.builder.build(&source_file, &self.build_dir)?;
        Ok(vec![source_file.path])
    }

    pub async fn create_code_archive(&self) -> Result<PathBuf, anyhow::Error> {
        Archiver::new(
            self.project_details.project_dir.clone(),
            self.build().await?,
        )
        .write()
    }
}
