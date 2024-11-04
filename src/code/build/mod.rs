use std::path::PathBuf;
use std::sync::Arc;

use crate::code::build::archive::write_archive;
use crate::code::build::swc::SwcBuilder;
use crate::code::parse::imports::ImportResolver;
use crate::code::parse::SourceParser;
use crate::code::source::path::{FunctionBuildDir, SourcePath};
use crate::code::source::{Language, ModuleImport, ModuleImports, SourceFile};
use crate::lambda::LambdaFn;
use crate::project::Lx3Project;

mod archive;
mod swc;

#[cfg(test)]
mod archive_test;
#[cfg(test)]
mod js_test;
#[cfg(test)]
mod swc_test;
#[cfg(test)]
mod ts_test;

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

pub struct LambdaBuildManifest {
    /// Absolute path to code.zip in FunctionBuildDir
    pub archive_path: PathBuf,
    /// Modules imported by LambdaFn
    pub fn_sources: Vec<SourcePath>,
}

pub struct LambdaFnBuild {
    build_dir: FunctionBuildDir,
    // todo arc
    builder: Box<dyn Builder + Send + Sync>,
    entrypoint: SourcePath,
    lambda_fn: Arc<LambdaFn>,
    project: Arc<Lx3Project>,
    source_parser: Arc<Box<dyn SourceParser>>,
}

impl LambdaFnBuild {
    pub fn in_api_dir(lambda_fn: Arc<LambdaFn>, project: Arc<Lx3Project>) -> Self {
        Self::new(
            FunctionBuildDir::PlatformSync(project.clone(), lambda_fn.clone()),
            lambda_fn,
            project,
        )
    }

    pub fn new(
        build_dir: FunctionBuildDir,
        lambda_fn: Arc<LambdaFn>,
        project: Arc<Lx3Project>,
    ) -> Self {
        let source_parser = {
            project
                .runtime_config
                .lock()
                .unwrap()
                .source_parser(&lambda_fn.language)
        };
        Self {
            build_dir,
            builder: Box::new(match &lambda_fn.language {
                Language::JavaScript | Language::TypeScript => SwcBuilder::new(project.clone()),
                Language::Python => panic!(),
            }),
            entrypoint: lambda_fn.path.clone(),
            lambda_fn,
            project,
            source_parser,
        }
    }

    pub async fn build(self) -> Result<LambdaBuildManifest, anyhow::Error> {
        let import_resolver = {
            self.project
                .runtime_config
                .lock()
                .unwrap()
                .import_resolver(&self.lambda_fn.language)
        };
        let fn_sources = parse_fn_sources(
            &self.entrypoint,
            self.source_parser.clone(),
            import_resolver,
        )
        .await?;
        let mut archive_sources = self.build_fn_sources(fn_sources.clone())?;
        archive_sources.append(&mut self.build_runtime_sources()?);
        let archive_path = write_archive(self.build_dir.to_path(), archive_sources.clone())?;
        Ok(LambdaBuildManifest {
            archive_path,
            fn_sources: archive_sources,
        })
    }

    // todo optimize with parallelism
    fn build_fn_sources(
        &self,
        fn_sources: Vec<SourcePath>,
    ) -> Result<Vec<SourcePath>, anyhow::Error> {
        let mut result = Vec::new();
        for fn_source in fn_sources {
            result.push(
                self.builder
                    .build(&self.source_parser.parse(fn_source)?, &self.build_dir)?,
            );
        }
        Ok(result)
    }

    // todo optimize with parallelism
    fn build_runtime_sources(&self) -> Result<Vec<SourcePath>, anyhow::Error> {
        let mut result = Vec::new();
        for runtime_source in self.get_runtime_sources() {
            if let Some(language) = runtime_source.language() {
                if language == self.lambda_fn.language {
                    result.push(self.builder.build(
                        &self.source_parser.parse(self.entrypoint.clone())?,
                        &self.build_dir,
                    )?);
                    continue;
                }
            }
            if runtime_source.abs().is_file() {
                result.push(runtime_source);
            }
        }
        Ok(result)
    }

    fn get_runtime_sources(&self) -> Vec<SourcePath> {
        self.project
            .runtime_config
            .lock()
            .unwrap()
            .runtime_sources(&self.lambda_fn.language)
    }
}

// todo recursively parse sources
// todo prevent circular dependency infinite loop
// todo optimize with parallelism
// todo LambdaFnBuild could be generified to have a runtime language's intermediary AST type as the
//  signature to build and parse APIs, optimizing a parse -> build workflow having to parse the AST
//  a second time for the build step
async fn parse_fn_sources(
    entrypoint_path: &SourcePath,
    source_parser: Arc<Box<dyn SourceParser>>,
    import_resolver: Arc<Box<dyn ImportResolver>>,
) -> Result<Vec<SourcePath>, anyhow::Error> {
    let entrypoint = source_parser.parse(entrypoint_path.clone())?;
    let mut result: Vec<SourcePath> = Vec::new();
    match entrypoint.imports {
        ModuleImports::Unprocessed(imports) => {
            for import in imports {
                match import_resolver.resolve(&entrypoint.path, import.as_str()) {
                    ModuleImport::RelativeSource(relative_source) => {
                        result.push(relative_source);
                    }
                    ModuleImport::Unknown(_) => panic!(),
                    ModuleImport::PackageDependency { .. } => todo!(),
                }
            }
        }
        _ => panic!(),
    }
    result.push(entrypoint.path);
    Ok(result)
}
