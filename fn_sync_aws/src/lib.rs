mod lambda_role;
mod sdk_clients;

use crate::sdk_clients::SdkClients;
use l3_fn_build::FnHandler;
use l3_fn_sync::{CloudPlatform, CreateFnError, LambdaFn};
use std::path::PathBuf;
use std::sync::Arc;

pub struct AwsCloudPlatform {
    sdk_clients: Arc<SdkClients>,
}

impl CloudPlatform for AwsCloudPlatform {
    fn initialize() -> Self {
        todo!()
    }

    fn create_fn(
        &self,
        entrypoint: PathBuf,
        handler: FnHandler,
    ) -> Result<LambdaFn, CreateFnError> {
        Ok(LambdaFn {})
    }
}
