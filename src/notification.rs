use std::sync::Arc;
use std::time::Instant;

use crate::lambda::LambdaFn;

pub enum LambdaNotification {
    LambdaEvent(LambdaEvent),
    LogEvent(LogEvent),
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
    UpdateCode(LambdaUpdateResult),
    UpdateDependencies(LambdaUpdateResult),
    UpdateEnv(LambdaUpdateResult),
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
