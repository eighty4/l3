use l3_fn_build::FnHandler;
use std::path::PathBuf;

pub struct LambdaFn {
    task_queue: TaskQueue,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateFnError {}

pub trait CloudPlatform {
    fn initialize() -> Self;
    fn create_fn(&self, entrypoint: PathBuf, handler: FnHandler)
        -> Result<LambdaFn, CreateFnError>;
}

impl LambdaFn {
    pub fn sync(&self) {}
}
