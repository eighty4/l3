use crate::task::launch::LambdaTaskLaunch;
use crate::task::translation::TaskTranslation;
use crate::task::{LambdaTask, LambdaTaskKind};
use create_fn::create_fn;
use delete_fn::delete_fn;
use update_code::update_code;
use update_env::update_env;

mod api;
mod create_fn;
mod delete_fn;
mod update_code;
mod update_env;

pub struct AwsTaskTranslation {}

impl TaskTranslation for AwsTaskTranslation {
    fn translate(&self, lambda_task: &LambdaTask) -> LambdaTaskLaunch {
        match lambda_task.kind {
            LambdaTaskKind::CreateFunction => create_fn,
            LambdaTaskKind::DeleteFunction => delete_fn,
            LambdaTaskKind::UpdateCode => update_code,
            LambdaTaskKind::UpdateEnvironment => update_env,
        }
    }
}
