use aws_config::SdkConfig;

#[derive(Clone)]
pub struct AwsClients {
    pub api_gateway: aws_sdk_apigatewayv2::Client,
    pub iam: aws_sdk_iam::Client,
    pub lambda: aws_sdk_lambda::Client,
}

impl From<SdkConfig> for AwsClients {
    fn from(config: SdkConfig) -> Self {
        Self {
            api_gateway: aws_sdk_apigatewayv2::Client::new(&config),
            iam: aws_sdk_iam::Client::new(&config),
            lambda: aws_sdk_lambda::Client::new(&config),
        }
    }
}
