use crate::lambda::RouteKey;
use tokio::sync::oneshot;

pub mod executor;
pub mod launch;
pub mod pool;
mod queue;
pub mod translation;

pub struct LambdaTask {
    pub completed: Option<oneshot::Sender<()>>,
    pub kind: LambdaTaskKind,
    pub route_key: RouteKey,
}

pub enum LambdaTaskKind {
    CreateFunction,
    DeleteFunction,
    UpdateCode,
    UpdateEnvironment,
}
