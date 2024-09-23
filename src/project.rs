use crate::aws::AwsProject;
use crate::code::build::BuildMode;
use crate::code::runtime::RuntimeConfig;
use crate::lambda::LambdaFn;
use crate::notification::{LambdaEvent, LambdaEventKind, LambdaNotification};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub struct Lx3Project {
    pub aws: AwsProject,
    pub build_mode: BuildMode,
    pub dir: PathBuf,
    pub name: String,
    pub notification_tx: UnboundedSender<LambdaNotification>,
    pub runtime_config: Arc<Mutex<RuntimeConfig>>,
}

impl Lx3Project {
    pub fn builder() -> Lx3ProjectBuilder {
        Lx3ProjectBuilder::new()
    }

    pub fn send_lambda_event(&self, lambda_fn: Arc<LambdaFn>, kind: LambdaEventKind) {
        self.notification_tx
            .send(LambdaNotification::Lambda(LambdaEvent { lambda_fn, kind }))
            .unwrap();
    }
}

#[derive(Default)]
pub struct Lx3ProjectBuilder {
    aws: Option<AwsProject>,
    build_mode: Option<BuildMode>,
    runtime_config: Option<Arc<Mutex<RuntimeConfig>>>,
}

impl Lx3ProjectBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn aws(mut self, aws: AwsProject) -> Self {
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

    pub fn build(
        self,
        project_dir: PathBuf,
        project_name: String,
    ) -> (Arc<Lx3Project>, UnboundedReceiver<LambdaNotification>) {
        debug_assert!(self.aws.is_some() && self.runtime_config.is_some());
        let (notification_tx, notification_rx) = unbounded_channel::<LambdaNotification>();
        (
            Arc::new(Lx3Project {
                aws: self.aws.unwrap(),
                build_mode: self.build_mode.unwrap_or(BuildMode::Debug),
                notification_tx,
                dir: project_dir,
                name: project_name,
                runtime_config: self.runtime_config.unwrap(),
            }),
            notification_rx,
        )
    }
}
