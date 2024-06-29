use std::collections::HashMap;

use anyhow::anyhow;
use aws_sdk_apigatewayv2::types::{Integration, Route};
use aws_sdk_lambda::types::FunctionConfiguration;

use crate::aws::clients::AwsClients;
use crate::aws::lambda::*;
use crate::lambda::{LambdaFn, RouteKey};

pub struct DeployedLambdaComponents {
    pub function_arn: Option<FunctionArn>,
    pub function_name: Option<FunctionName>,
    pub integration: Option<(IntegrationId, FunctionArn)>,
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
    pub async fn fetch_from_aws(
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

    #[allow(unused)]
    pub fn get_deployed_components(&self, lambda_fn: &LambdaFn) -> DeployedLambdaComponents {
        let route = self.routes.get(&lambda_fn.route_key);
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
        let function = self.functions.get(&lambda_fn.fn_name);
        DeployedLambdaComponents {
            function_arn: function.and_then(|f| f.function_arn.clone()),
            function_name: function.and_then(|f| f.function_name.clone()),
            integration: integration.map(|i| {
                (
                    i.integration_id.clone().unwrap(),
                    i.integration_uri.clone().unwrap(),
                )
            }),
            route: route.and_then(|r| r.route_id.clone()),
        }
    }

    pub fn rm_deployed_components(
        &mut self,
        route_key: &RouteKey,
        fn_name: &String,
    ) -> DeployedLambdaComponents {
        let route = self.routes.remove(route_key);
        let integration = match &route {
            None => None,
            Some(route) => {
                let integration_id = route
                    .target
                    .as_ref()
                    .and_then(|target| target.strip_prefix("integrations/"));
                integration_id.and_then(|integration_id| self.integrations.remove(integration_id))
            }
        };
        let maybe_function = self.functions.remove(fn_name);
        let (function_arn, function_name) = match maybe_function {
            None => (None, None),
            Some(fn_config) => (fn_config.function_arn.clone(), fn_config.function_name),
        };
        DeployedLambdaComponents {
            function_arn,
            function_name,
            integration: integration.map(|i| {
                (
                    i.integration_id.clone().unwrap(),
                    i.integration_uri.clone().unwrap(),
                )
            }),
            route: route.and_then(|r| r.route_id),
        }
    }

    pub fn collect_deployed_components(
        mut self,
        project_name: &String,
    ) -> Vec<DeployedLambdaComponents> {
        let mut components = Vec::new();
        // todo does routes.keys() have to be cloned to use &mut self within the for loop?
        for route_key in self.routes.keys().cloned().collect::<Vec<RouteKey>>() {
            let fn_name = route_key.to_fn_name(project_name);
            components.push(self.rm_deployed_components(&route_key, &fn_name));
        }
        // todo does integrations.keys() have to be cloned to use &mut self within the for loop?
        for integration_id in self
            .integrations
            .keys()
            .cloned()
            .collect::<Vec<IntegrationId>>()
        {
            let removed_integration = self.integrations.remove(&integration_id);
            let fn_name = removed_integration
                .clone()
                .and_then(|i| i.integration_uri)
                .map(|fn_arn| parse_fn_name_from_arn(&fn_arn));
            let maybe_fn = fn_name.and_then(|fn_name| self.functions.remove(&fn_name));
            let (function_arn, function_name) = match maybe_fn {
                None => (None, None),
                Some(fn_config) => (fn_config.function_name.clone(), fn_config.function_name),
            };
            let integration = removed_integration
                .map(|i| (i.integration_id.unwrap(), i.integration_uri.unwrap()));
            components.push(DeployedLambdaComponents {
                function_name,
                function_arn,
                integration,
                route: None,
            });
        }
        components.append(
            &mut self
                .functions
                .into_values()
                .map(|fn_config| DeployedLambdaComponents {
                    integration: None,
                    route: None,
                    function_arn: fn_config.function_arn.clone(),
                    function_name: fn_config.function_name,
                })
                .collect(),
        );
        assert!(self.routes.is_empty());
        assert!(self.integrations.is_empty());
        components
    }
}
