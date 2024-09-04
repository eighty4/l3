use crate::lambda::LambdaFn;
use std::sync::Arc;
use std::time::Instant;

pub enum LambdaNotification {
    Lambda(LambdaEvent),
    Log(LogEvent),
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
    Dependencies,
    Env,
}

pub enum LambdaUpdateResult {
    Success,
    Failure(String),
}

pub struct LogEvent {
    pub lambda_fn: Arc<LambdaFn>,
    pub message: String,
    pub timestamp: Instant,
}

pub enum SourcesEvent {}
