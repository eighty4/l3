use crate::task::launch::LambdaTaskLaunch;
use crate::task::LambdaTask;

pub trait TaskTranslation {
    fn translate(&self, lambda_task: &LambdaTask) -> LambdaTaskLaunch;
}
