use crate::task::workflow::{TaskContinuation, TaskWorkflow};

pub enum AwsTaskKind {
    CreateFunction,
    PublishDependencyLayer,
    RemoveFunction,
    UpdateCode,
    UpdateEnv,
    UpdateLayer,
}

pub struct AwsTaskWorkflow {}

impl AwsTaskWorkflow {
    pub fn new() -> Self {
        Self {}
    }
}

impl TaskWorkflow<AwsTaskKind> for AwsTaskWorkflow {
    fn on_completed_task(task: AwsTaskKind) -> TaskContinuation<AwsTaskKind> {
        match task {
            AwsTaskKind::PublishDependencyLayer => {
                // todo create an UpdateLayer task for every lambda
                TaskContinuation::QueueTasks(vec![])
            }
            _ => TaskContinuation::Finished,
        }
    }

    fn on_failed_task(_: AwsTaskKind) -> TaskContinuation<AwsTaskKind> {
        TaskContinuation::Finished
    }
}
