use anyhow::anyhow;
use std::collections::HashMap;

use aws_sdk_apigatewayv2::types::{Integration, Route};
use aws_sdk_lambda::types::FunctionConfiguration;

use crate::aws::clients::AwsClients;
use crate::aws::lambda::{FunctionArn, FunctionName, IntegrationId, RouteId};
use crate::lambda::RouteKey;

pub struct DeployedLambdaComponents {
    pub function: Option<FunctionArn>,
    pub integration: Option<IntegrationId>,
    pub route: Option<RouteId>,
}

// todo diffing deployed components mutably, to remove dangling resources after sync
pub struct DeployedProjectState {
    pub functions: HashMap<FunctionName, FunctionConfiguration>,
    pub integrations: HashMap<IntegrationId, Integration>,
    pub routes: HashMap<RouteKey, Route>,
}

impl DeployedProjectState {
    pub fn new(
        project_name: &String,
        fetched_functions: Vec<FunctionConfiguration>,
        fetched_integrations: Vec<Integration>,
        fetched_routes: Vec<Route>,
    ) -> Self {
        let fn_prefix = format!("l3-{project_name}-");
        let mut functions = HashMap::new();
        for function in fetched_functions {
            if function
                .function_name
                .as_ref()
                .map_or(false, |fn_name| fn_name.starts_with(fn_prefix.as_str()))
            {
                functions.insert(function.function_name.clone().unwrap(), function);
            }
        }
        let mut routes = HashMap::new();
        for route in fetched_routes {
            let route_key = RouteKey::try_from(route.route_key.clone().unwrap()).unwrap();
            routes.insert(route_key, route);
        }
        let mut integrations = HashMap::new();
        for integration in fetched_integrations {
            integrations.insert(integration.integration_id.clone().unwrap(), integration);
        }
        Self {
            functions,
            integrations,
            routes,
        }
    }

    // todo handle pagination across multiple requests for functions, integrations and routes
    pub async fn fetch_state_from_aws(
        sdk_clients: &AwsClients,
        project_name: &String,
        api_id: &String,
    ) -> Result<Self, anyhow::Error> {
        let functions = sdk_clients
            .lambda
            .list_functions()
            .send()
            .await
            .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?
            .functions
            .unwrap();
        let routes = sdk_clients
            .api_gateway
            .get_routes()
            .api_id(api_id)
            .send()
            .await
            .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?
            .items
            .unwrap();
        let integrations = sdk_clients
            .api_gateway
            .get_integrations()
            .api_id(api_id)
            .send()
            .await
            .map_err(|err| anyhow!("{}", err.into_service_error().to_string()))?
            .items
            .unwrap();
        Ok(DeployedProjectState::new(
            project_name,
            functions,
            integrations,
            routes,
        ))
    }

    pub fn get_deployed_components(
        &self,
        fn_name: &String,
        route_key: &RouteKey,
    ) -> DeployedLambdaComponents {
        let route = self.routes.get(route_key);
        let integration = match route {
            None => None,
            Some(route) => {
                let integration_id = route
                    .target
                    .as_ref()
                    .and_then(|target| target.strip_prefix("integrations/"));
                integration_id.and_then(|integration_id| self.integrations.get(integration_id))
            }
        };
        let function = self.functions.get(fn_name);
        DeployedLambdaComponents {
            function: function.and_then(|f| f.function_arn.clone()),
            integration: integration.and_then(|i| i.integration_id.clone()),
            route: route.and_then(|r| r.route_id.clone()),
        }
    }
}
