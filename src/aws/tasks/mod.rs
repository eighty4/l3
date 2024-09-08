use crate::aws::tasks::dev::create_fn::create_fn;
use crate::aws::tasks::dev::delete_fn::delete_fn;
use crate::aws::tasks::dev::update_code::update_code;
use crate::aws::tasks::dev::update_env::update_env;
use crate::task::launch::LambdaTaskLaunch;
use crate::task::translation::TaskTranslation;
use crate::task::{LambdaTask, LambdaTaskKind};

mod api;
mod dev;
pub mod sync;

pub struct AwsTaskTranslation {}

impl TaskTranslation for AwsTaskTranslation {
    fn translate(&self, lambda_task: LambdaTask) -> LambdaTaskLaunch {
        match lambda_task.kind {
            LambdaTaskKind::CreateFunction => create_fn,
            LambdaTaskKind::DeleteFunction => delete_fn,
            LambdaTaskKind::UpdateCode => update_code,
            LambdaTaskKind::UpdateEnvironment => update_env,
        }
    }
}
