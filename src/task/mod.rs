use crate::lambda::RouteKey;

pub mod executor;
pub mod launch;
pub mod pool;
mod queue;
pub mod translation;

pub struct LambdaTask {
    pub kind: LambdaTaskKind,
    pub route_key: RouteKey,
}

pub enum LambdaTaskKind {
    CreateFunction,
    DeleteFunction,
    UpdateCode,
    UpdateEnvironment,
}
