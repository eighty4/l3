use std::path::PathBuf;

use crate::aws::AwsDeets;
use crate::code::build::BuildMode;
use crate::code::runtime::SourcesRuntimeDeets;

pub struct Lx3ProjectDeets {
    pub aws: AwsDeets,
    pub build_mode: BuildMode,
    pub project_dir: PathBuf,
    pub project_name: String,
    pub sources: SourcesRuntimeDeets,
}

impl Lx3ProjectDeets {
    pub fn builder() -> Lx3ProjectDeetsBuilder {
        Lx3ProjectDeetsBuilder::new()
    }
}

#[derive(Default)]
pub struct Lx3ProjectDeetsBuilder {
    aws: Option<AwsDeets>,
    build_mode: Option<BuildMode>,
    runtime_deets: Option<SourcesRuntimeDeets>,
}

impl Lx3ProjectDeetsBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn aws_deets(mut self, aws: AwsDeets) -> Self {
        self.aws = Some(aws);
        self
    }

    pub fn build_mode(mut self, build_mode: BuildMode) -> Self {
        self.build_mode = Some(build_mode);
        self
    }

    pub fn runtime_deets(mut self, runtime_deets: SourcesRuntimeDeets) -> Self {
        self.runtime_deets = Some(runtime_deets);
        self
    }

    pub fn build(self, project_dir: PathBuf, project_name: String) -> Lx3ProjectDeets {
        debug_assert!(self.aws.is_some() && self.runtime_deets.is_some());
        Lx3ProjectDeets {
            aws: self.aws.unwrap(),
            build_mode: self.build_mode.unwrap_or(BuildMode::Debug),
            project_dir,
            project_name,
            sources: self.runtime_deets.unwrap(),
        }
    }
}
