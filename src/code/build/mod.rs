use std::sync::Arc;

use crate::code::build::archiver::Archive;
use crate::code::build::swc::SwcBuilder;
use crate::code::parse::SourceParser;
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
    lambda_fn: Arc<LambdaFn>,
    project_details: Arc<Lx3ProjectDeets>,
    source_parser: Arc<Box<dyn SourceParser>>,
}

impl LambdaFnBuild {
    pub fn new(lambda_fn: Arc<LambdaFn>, project_details: Arc<Lx3ProjectDeets>) -> Self {
        let source_parser = {
            project_details
                .runtime_config
                .lock()
                .unwrap()
                .source_parser(&lambda_fn.language)
        };
        Self {
            build_dir: lambda_fn.build_dir(&project_details),
            builder: Box::new(match &lambda_fn.language {
                Language::JavaScript | Language::TypeScript => {
                    SwcBuilder::new(project_details.clone())
                }
                Language::Python => panic!(),
            }),
            entrypoint: lambda_fn.path.clone(),
            lambda_fn,
            project_details,
            source_parser,
        }
    }

    pub async fn build(&self) -> Result<Vec<SourcePath>, anyhow::Error> {
        let mut sources = vec![self.builder.build(
            &self.source_parser.parse(self.entrypoint.clone())?,
            &self.build_dir,
        )?];
        let runtime_sources = {
            self.project_details
                .runtime_config
                .lock()
                .unwrap()
                .runtime_sources(&self.lambda_fn.language)
        };
        for runtime_source in runtime_sources {
            if let Some(language) = runtime_source.language() {
                if language == self.lambda_fn.language {
                    sources.push(self.builder.build(
                        &self.source_parser.parse(self.entrypoint.clone())?,
                        &self.build_dir,
                    )?);
                    continue;
                }
            }
            if runtime_source.abs.is_file() {
                sources.push(runtime_source);
            }
        }
        Ok(sources)
    }

    pub async fn create_code_archive(&self) -> Result<Archive, anyhow::Error> {
        Archiver::new(self.build_dir.abs.clone(), self.build().await?).write()
    }
}
