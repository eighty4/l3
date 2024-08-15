use crate::lambda::LambdaFn;
use std::sync::Arc;
use std::time::Instant;

pub enum LambdaNotification {
    Lambda(LambdaEvent),
    Log(LogEvent),
    Sources(SourcesEvent),
}

pub struct LambdaEvent {
    lambda_fn: Arc<LambdaFn>,
    kind: LambdaEventKind,
}

pub enum LambdaEventKind {
    Creating,
    Created(LambdaUpdateResult),
    Removing,
    Removed(LambdaUpdateResult),
    Updating(LambdaUpdateKind),
    Updated(LambdaUpdateKind),
}

pub enum LambdaUpdateKind {
    Code(LambdaUpdateResult),
    Dependencies(LambdaUpdateResult),
    Env(LambdaUpdateResult),
}

pub enum LambdaUpdateResult {
    Success,
    Failure(String),
}

pub struct LogEvent {
    lambda_fn: Arc<LambdaFn>,
    message: String,
    timestamp: Instant,
}

pub enum SourcesEvent {}
