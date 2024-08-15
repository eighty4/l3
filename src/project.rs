use crate::aws::AwsDeets;
use crate::code::build::BuildMode;
use crate::code::runtime::RuntimeConfig;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct Lx3ProjectDeets {
    pub aws: AwsDeets,
    pub build_mode: BuildMode,
    pub project_dir: PathBuf,
    pub project_name: String,
    pub runtime_config: Arc<Mutex<RuntimeConfig>>,
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
    runtime_config: Option<Arc<Mutex<RuntimeConfig>>>,
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

    pub fn runtime_config(mut self, runtime_config: Arc<Mutex<RuntimeConfig>>) -> Self {
        self.runtime_config = Some(runtime_config);
        self
    }

    pub fn build(self, project_dir: PathBuf, project_name: String) -> Lx3ProjectDeets {
        debug_assert!(self.aws.is_some() && self.runtime_config.is_some());
        Lx3ProjectDeets {
            aws: self.aws.unwrap(),
            build_mode: self.build_mode.unwrap_or(BuildMode::Debug),
            project_dir,
            project_name,
            runtime_config: self.runtime_config.unwrap(),
        }
    }
}
