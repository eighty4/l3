use crate::lambda::LambdaFn;
use crate::project::Lx3ProjectDeets;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type LaunchedTask = Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>>;

pub type LambdaTaskLaunch =
    fn(project_deets: Arc<Lx3ProjectDeets>, lambda_fn: Arc<LambdaFn>) -> LaunchedTask;
