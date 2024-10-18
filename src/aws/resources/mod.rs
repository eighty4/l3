use crate::aws::resources::runtime::AwsLambdaRuntime;
use crate::lambda::RouteKey;
use aws_sdk_apigatewayv2::operation::create_integration::CreateIntegrationOutput;
use aws_sdk_apigatewayv2::operation::create_route::CreateRouteOutput;
use aws_sdk_apigatewayv2::operation::update_integration::UpdateIntegrationOutput;
use aws_sdk_apigatewayv2::operation::update_route::UpdateRouteOutput;
use aws_sdk_apigatewayv2::types::{Integration, Route};
use aws_sdk_lambda::operation::create_function::CreateFunctionOutput;
use aws_sdk_lambda::operation::update_function_configuration::UpdateFunctionConfigurationOutput;
use aws_sdk_lambda::types::FunctionConfiguration;
use std::collections::HashMap;
use std::sync::Arc;

mod fetch;
pub(crate) mod repository;
pub(crate) mod runtime;
mod state;

#[cfg(test)]
mod resources_test;

#[cfg(test)]
mod state_test;

pub type FunctionArn = String;
pub type FunctionName = String;
pub type IntegrationId = String;
pub type RouteId = String;

#[allow(unused)]
pub fn parse_fn_name_from_arn(fn_arn: &FunctionArn) -> FunctionName {
    fn_arn.split(':').last().unwrap().to_string()
}

pub struct AwsLambdaFunction {
    pub arn: FunctionArn,
    pub env: Option<HashMap<String, String>>,
    #[allow(unused)]
    pub handler: String,
    pub name: FunctionName,
    #[allow(unused)]
    pub role: String,
    #[allow(unused)]
    pub runtime: AwsLambdaRuntime,
}

pub enum AwsGatewayRouteTarget {
    GatewayIntegration(IntegrationId),
    #[allow(unused)]
    Unknown(String),
}

pub struct AwsGatewayIntegration {
    pub id: IntegrationId,
    pub integration_uri: FunctionArn,
}

pub struct AwsGatewayRoute {
    pub id: RouteId,
    pub route_key: RouteKey,
    pub target: AwsGatewayRouteTarget,
}

pub struct AwsLambdaResources {
    pub function: Option<Arc<AwsLambdaFunction>>,
    pub integration: Option<Arc<AwsGatewayIntegration>>,
    pub route: Option<Arc<AwsGatewayRoute>>,
}

impl From<CreateFunctionOutput> for AwsLambdaFunction {
    fn from(v: CreateFunctionOutput) -> Self {
        Self {
            arn: v.function_arn.unwrap(),
            env: v.environment.and_then(|e| e.variables),
            handler: v.handler.unwrap(),
            name: v.function_name.unwrap(),
            role: v.role.unwrap(),
            runtime: AwsLambdaRuntime::from(v.runtime.unwrap()),
        }
    }
}

impl From<FunctionConfiguration> for AwsLambdaFunction {
    fn from(v: FunctionConfiguration) -> Self {
        Self {
            arn: v.function_arn.unwrap(),
            env: v.environment.and_then(|e| e.variables),
            handler: v.handler.unwrap(),
            name: v.function_name.unwrap(),
            role: v.role.unwrap(),
            runtime: AwsLambdaRuntime::from(v.runtime.unwrap()),
        }
    }
}

impl From<UpdateFunctionConfigurationOutput> for AwsLambdaFunction {
    fn from(v: UpdateFunctionConfigurationOutput) -> Self {
        Self {
            arn: v.function_arn.unwrap(),
            env: v.environment.and_then(|e| e.variables),
            handler: v.handler.unwrap(),
            name: v.function_name.unwrap(),
            role: v.role.unwrap(),
            runtime: AwsLambdaRuntime::from(v.runtime.unwrap()),
        }
    }
}

impl From<String> for AwsGatewayRouteTarget {
    fn from(target: String) -> Self {
        match target.strip_prefix("integrations/") {
            None => AwsGatewayRouteTarget::Unknown(target),
            Some(integration_id) => {
                AwsGatewayRouteTarget::GatewayIntegration(integration_id.to_string())
            }
        }
    }
}

impl From<CreateRouteOutput> for AwsGatewayRoute {
    fn from(v: CreateRouteOutput) -> Self {
        Self {
            id: v.route_id.unwrap(),
            route_key: RouteKey::from_route_key_string(v.route_key.unwrap()).unwrap(),
            target: AwsGatewayRouteTarget::from(v.target.unwrap()),
        }
    }
}

impl From<Route> for AwsGatewayRoute {
    fn from(v: Route) -> Self {
        Self {
            id: v.route_id.unwrap(),
            route_key: RouteKey::from_route_key_string(v.route_key.unwrap()).unwrap(),
            target: AwsGatewayRouteTarget::from(v.target.unwrap()),
        }
    }
}

impl From<UpdateRouteOutput> for AwsGatewayRoute {
    fn from(v: UpdateRouteOutput) -> Self {
        Self {
            id: v.route_id.unwrap(),
            route_key: RouteKey::from_route_key_string(v.route_key.unwrap()).unwrap(),
            target: AwsGatewayRouteTarget::from(v.target.unwrap()),
        }
    }
}

impl From<CreateIntegrationOutput> for AwsGatewayIntegration {
    fn from(v: CreateIntegrationOutput) -> Self {
        Self {
            id: v.integration_id.unwrap(),
            integration_uri: v.integration_uri.unwrap(),
        }
    }
}

impl From<Integration> for AwsGatewayIntegration {
    fn from(v: Integration) -> Self {
        Self {
            id: v.integration_id.unwrap(),
            integration_uri: v.integration_uri.unwrap(),
        }
    }
}

impl From<UpdateIntegrationOutput> for AwsGatewayIntegration {
    fn from(v: UpdateIntegrationOutput) -> Self {
        Self {
            id: v.integration_id.unwrap(),
            integration_uri: v.integration_uri.unwrap(),
        }
    }
}
