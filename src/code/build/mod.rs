use std::path::PathBuf;

use crate::code::archiver::Archiver;
use crate::code::build::swc::SwcBuilder;
use crate::code::parse::parse_source_file;
use crate::code::project::ProjectDetails;
use crate::code::source::path::SourcePath;
use crate::code::source::{FunctionBuildDir, Language, SourceFile};
use crate::lambda::LambdaFn;

mod swc;

#[cfg(test)]
mod swc_test;

trait Builder {
    fn build(
        &self,
        source_file: &SourceFile,
        options: &BuildOptions,
    ) -> Result<SourcePath, anyhow::Error>;
}

#[derive(Clone, PartialEq)]
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

pub struct BuildOptions {
    build_dir: FunctionBuildDir,
    mode: BuildMode,
    project_dir: PathBuf,
}

impl BuildOptions {
    fn new(build_dir: FunctionBuildDir, mode: BuildMode, project_dir: PathBuf) -> Self {
        Self {
            build_dir,
            mode,
            project_dir,
        }
    }
}

pub struct LambdaFnBuild {
    builder: Box<dyn Builder + Send + Sync>,
    entrypoint: SourcePath,
    options: BuildOptions,
    project_details: ProjectDetails,
}

impl LambdaFnBuild {
    pub fn new(
        project_details: ProjectDetails,
        project_dir: PathBuf,
        api_id: String,
        build_mode: BuildMode,
        lambda_fn: LambdaFn,
    ) -> Self {
        let build_dir =
            FunctionBuildDir::new(api_id, build_mode.clone(), lambda_fn.fn_name.clone());
        Self {
            builder: Box::new(match &lambda_fn.language {
                Language::JavaScript | Language::TypeScript => SwcBuilder::new(),
                Language::Python => panic!(),
            }),
            entrypoint: lambda_fn.path.clone(),
            options: BuildOptions::new(build_dir, build_mode, project_dir),
            project_details,
        }
    }

    pub async fn build(&self) -> Result<Vec<SourcePath>, anyhow::Error> {
        let source_file = parse_source_file(self.entrypoint.clone(), &self.project_details)?;
        self.builder.build(&source_file, &self.options)?;
        Ok(vec![source_file.path])
    }

    pub async fn create_code_archive(&self) -> Result<PathBuf, anyhow::Error> {
        Archiver::new(&self.options.project_dir, self.dir(), self.build().await?).write()
    }

    pub fn dir(&self) -> &FunctionBuildDir {
        &self.options.build_dir
    }
}
