use deploy_fn::perform_deploy_fn;

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
    pub deployed_components: DeployedLambdaComponents,
    pub lambda_fn: LambdaFn,
    pub lambda_role_arn: String,
    pub region: String,
    pub stage_name: String,
}

pub enum SyncTask {
    DeployFn(DeployFnParams),
    #[allow(unused)]
    RemoveFn(DeployedLambdaComponents),
    #[allow(unused)]
    UpdateCode(LambdaFn),
    #[allow(unused)]
    UpdateEnv(LambdaFn),
}

pub async fn exec(sdk_clients: &AwsClients, task: SyncTask) -> Result<(), anyhow::Error> {
    if let SyncTask::DeployFn(params) = task {
        perform_deploy_fn(sdk_clients, params).await?;
    }
    Ok(())
}
