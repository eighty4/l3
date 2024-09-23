use anyhow::anyhow;
use aws_config::meta::region::RegionProviderChain;
use aws_config::{AppName, BehaviorVersion, Region, SdkConfig};
use aws_sdk_iam::config::ProvideCredentials;
use std::sync::Arc;

pub struct AwsClients {
    pub api_gateway: aws_sdk_apigatewayv2::Client,
    pub iam: aws_sdk_iam::Client,
    pub lambda: aws_sdk_lambda::Client,
    sdk_config: SdkConfig,
}

impl AwsClients {
    pub async fn new(project_name: &str) -> Result<Arc<AwsClients>, anyhow::Error> {
        Ok(Arc::new(Self::from(
            aws_config::defaults(BehaviorVersion::v2024_03_28())
                .app_name(AppName::new(Self::create_app_name(project_name))?)
                .region(RegionProviderChain::default_provider().or_else("us-east-1"))
                .load()
                .await,
        )))
    }

    pub async fn expect_credentials(&self) -> Result<(), anyhow::Error> {
        match self
            .sdk_config
            .credentials_provider()
            .expect("aws credentials provider")
            .provide_credentials()
            .await
        {
            Ok(_) => {
                // todo verify roles for l3 operations
                //  api gateway
                //  iam
                //  lambda
                Ok(())
            }
            Err(err) => Err(anyhow!("aws credentials error: {err}")),
        }
    }

    pub fn region(&self) -> Region {
        self.sdk_config
            .region()
            .cloned()
            .expect("aws configured region")
    }

    fn create_app_name(project_name: &str) -> String {
        if "l3-"
            == project_name
                .chars()
                .take(3)
                .collect::<String>()
                .to_lowercase()
        {
            project_name.to_string()
        } else {
            format!("l3-{project_name}")
        }
    }
}

impl From<SdkConfig> for AwsClients {
    fn from(sdk_config: SdkConfig) -> Self {
        Self {
            api_gateway: aws_sdk_apigatewayv2::Client::new(&sdk_config),
            iam: aws_sdk_iam::Client::new(&sdk_config),
            lambda: aws_sdk_lambda::Client::new(&sdk_config),
            sdk_config,
        }
    }
}
