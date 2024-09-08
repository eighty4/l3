use crate::code::source::tree::SourceTree;
use crate::notification::LambdaNotification;
use crate::project::Lx3ProjectDeets;
use crate::task::translation::TaskTranslation;
use crate::task::LambdaTask;
use std::sync::{Arc, Mutex};
use tokio::spawn;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

pub struct TaskExecutor {
    notification_tx: UnboundedSender<LambdaNotification>,
    project_deets: Arc<Lx3ProjectDeets>,
    source_tree: Arc<Mutex<SourceTree>>,
    task_translation: Box<dyn TaskTranslation + Send + Sync>,
}

impl TaskExecutor {
    pub fn new(
        notification_tx: UnboundedSender<LambdaNotification>,
        project_deets: Arc<Lx3ProjectDeets>,
        source_tree: Arc<Mutex<SourceTree>>,
        task_translation: Box<dyn TaskTranslation + Send + Sync>,
    ) -> Self {
        Self {
            notification_tx,
            project_deets,
            source_tree,
            task_translation,
        }
    }

    pub async fn start_task(
        &self,
        lambda_task: LambdaTask,
    ) -> JoinHandle<Result<(), anyhow::Error>> {
        let notification_tx = self.notification_tx.clone();
        let project_deets = self.project_deets.clone();
        let lambda_fn = self
            .source_tree
            .lock()
            .unwrap()
            .lambda_fn_by_route_key(&lambda_task.route_key)
            .unwrap();
        let task_launch = self.task_translation.translate(lambda_task);
        spawn(async move { task_launch(notification_tx, project_deets, lambda_fn).await })
    }
}
