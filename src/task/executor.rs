use crate::code::source::tree::SourceTree;
use crate::project::Lx3Project;
use crate::task::translation::TaskTranslation;
use crate::task::LambdaTask;
use std::sync::{Arc, Mutex};
use tokio::spawn;
use tokio::task::JoinHandle;

pub struct TaskExecutor {
    project: Arc<Lx3Project>,
    source_tree: Arc<Mutex<SourceTree>>,
    task_translation: Box<dyn TaskTranslation + Send + Sync>,
}

impl TaskExecutor {
    pub fn new(
        project: Arc<Lx3Project>,
        source_tree: Arc<Mutex<SourceTree>>,
        task_translation: Box<dyn TaskTranslation + Send + Sync>,
    ) -> Self {
        Self {
            project,
            source_tree,
            task_translation,
        }
    }

    pub fn start_lambda_task(
        &self,
        lambda_task: &LambdaTask,
    ) -> JoinHandle<Result<(), anyhow::Error>> {
        let project = self.project.clone();
        let lambda_fn = self
            .source_tree
            .lock()
            .unwrap()
            .lambda_fn_by_route_key(&lambda_task.route_key)
            .unwrap();
        let task_launch = self.task_translation.translate(lambda_task);
        spawn(async move { task_launch(project, lambda_fn).await })
    }
}
