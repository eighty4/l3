use crate::lambda::LambdaFn;
use std::sync::Arc;
use std::time::Instant;

pub enum LambdaNotification {
    Lambda(LambdaEvent),
    #[allow(unused)]
    Log(LogEvent),
    #[allow(unused)]
    Sources(SourcesEvent),
}

pub struct LambdaEvent {
    pub lambda_fn: Arc<LambdaFn>,
    pub kind: LambdaEventKind,
}

pub enum LambdaEventKind {
    Creating,
    Created(LambdaUpdateResult),
    Removing,
    Removed(LambdaUpdateResult),
    Updating(LambdaUpdateKind),
    Updated(LambdaUpdateKind, LambdaUpdateResult),
}

pub enum LambdaUpdateKind {
    Code,
    #[allow(unused)]
    Dependencies,
    Env,
}

pub enum LambdaUpdateResult {
    Failure(String),
    Success,
}

pub struct LogEvent {
    pub lambda_fn: Arc<LambdaFn>,
    pub message: String,
    pub timestamp: Instant,
}

pub enum SourcesEvent {}
