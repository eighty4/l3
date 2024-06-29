use tokio::task::JoinSet;

use deploy_fn::perform_deploy_fn;
use remove_fn::perform_remove_fn;

use crate::aws::clients::AwsClients;
use crate::aws::state::DeployedLambdaComponents;
use crate::lambda::LambdaFn;

mod deploy_fn;
mod remove_fn;
mod update_code;
mod update_env;

pub struct DeployFnParams {
    pub account_id: String,
    pub api_id: String,
    pub components: DeployedLambdaComponents,
    pub lambda_fn: LambdaFn,
    pub lambda_role_arn: String,
    pub publish_fn_updates: bool,
    pub region: String,
    pub stage_name: String,
}

pub struct RemoveFnParams {
    pub api_id: String,
    pub components: DeployedLambdaComponents,
}

pub enum SyncTask {
    DeployFn(Box<DeployFnParams>),
    RemoveFn(Box<RemoveFnParams>),
}

pub async fn exec(
    sdk_clients: &AwsClients,
    sync_tasks: Vec<SyncTask>,
) -> Result<(), anyhow::Error> {
    let mut join_set = JoinSet::new();
    for sync_task in sync_tasks {
        join_set.spawn(exec_task(sdk_clients.clone(), sync_task));
    }
    while let Some(result) = join_set.join_next().await {
        // todo handle sync errors
        result??;
    }
    Ok(())
}

async fn exec_task(sdk_clients: AwsClients, sync_task: SyncTask) -> Result<(), anyhow::Error> {
    match sync_task {
        SyncTask::DeployFn(params) => perform_deploy_fn(&sdk_clients, params.as_ref()).await?,
        SyncTask::RemoveFn(params) => perform_remove_fn(&sdk_clients, params.as_ref()).await?,
    }
    Ok::<(), anyhow::Error>(())
}
