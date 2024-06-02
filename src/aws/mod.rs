use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, SdkConfig};

pub(crate) mod iam;
pub(crate) mod lambda;

pub async fn load_sdk_config() -> SdkConfig {
    aws_config::defaults(BehaviorVersion::v2024_03_28())
        .region(RegionProviderChain::default_provider().or_else("us-east-1"))
        .load()
        .await
}
