use crate::aws::clients::AwsClients;
use crate::aws::resources::fetch::fetch_project_state;
use crate::aws::resources::{
    AwsGatewayIntegration, AwsGatewayRoute, AwsGatewayRouteTarget, AwsLambdaFunction,
    AwsLambdaResources, FunctionName, IntegrationId,
};
use crate::aws::AwsApiGateway;
use crate::lambda::{LambdaFn, RouteKey};
use aws_sdk_apigatewayv2::types::{Integration, Route};
use aws_sdk_lambda::types::FunctionConfiguration;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct DeployedProjectState {
    pub functions: HashMap<FunctionName, Arc<AwsLambdaFunction>>,
    pub integrations: HashMap<IntegrationId, Arc<AwsGatewayIntegration>>,
    pub routes: HashMap<RouteKey, Arc<AwsGatewayRoute>>,
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
                functions.insert(
                    function.function_name.clone().unwrap(),
                    Arc::new(AwsLambdaFunction::from(function)),
                );
            }
        }
        let mut routes = HashMap::new();
        for route in fetched_routes {
            let route_key =
                RouteKey::from_route_key_string(route.route_key.clone().unwrap()).unwrap();
            routes.insert(route_key, Arc::new(AwsGatewayRoute::from(route)));
        }
        let mut integrations = HashMap::new();
        for integration in fetched_integrations {
            integrations.insert(
                integration.integration_id.clone().unwrap(),
                Arc::new(AwsGatewayIntegration::from(integration)),
            );
        }
        Self {
            functions,
            integrations,
            routes,
        }
    }

    pub async fn fetch_from_aws(
        sdk_clients: &Arc<AwsClients>,
        project_name: &String,
        api_gateway: &Arc<AwsApiGateway>,
    ) -> Result<Self, anyhow::Error> {
        let (functions, integrations, routes) =
            fetch_project_state(sdk_clients, api_gateway).await?;
        Ok(DeployedProjectState::new(
            project_name,
            functions,
            integrations,
            routes,
        ))
    }

    pub fn created_lambda_function(&mut self, lambda_config: Arc<AwsLambdaFunction>) {
        self.functions
            .insert(lambda_config.name.clone(), lambda_config);
    }

    pub fn created_gateway_integration(&mut self, gateway_integration: Arc<AwsGatewayIntegration>) {
        self.integrations
            .insert(gateway_integration.id.clone(), gateway_integration);
    }

    pub fn created_gateway_route(&mut self, gateway_route: Arc<AwsGatewayRoute>) {
        self.routes
            .insert(gateway_route.route_key.clone(), gateway_route);
    }

    pub fn deleted_lambda_function(&mut self, lambda_config: Arc<AwsLambdaFunction>) {
        self.functions.remove(&lambda_config.name);
    }

    pub fn deleted_gateway_integration(&mut self, gateway_integration: Arc<AwsGatewayIntegration>) {
        self.integrations.remove(&gateway_integration.id);
    }

    pub fn deleted_gateway_route(&mut self, gateway_route: Arc<AwsGatewayRoute>) {
        self.routes.remove(&gateway_route.route_key);
    }

    pub fn get_deployed_components(&self, lambda_fn: &LambdaFn) -> AwsLambdaResources {
        let route = self.routes.get(&lambda_fn.route_key).cloned();
        let integration = match &route {
            None => None,
            Some(route) => {
                if let AwsGatewayRouteTarget::GatewayIntegration(integration_id) = &route.target {
                    self.integrations.get(integration_id).cloned()
                } else {
                    None
                }
            }
        };
        AwsLambdaResources {
            function: self.functions.get(&lambda_fn.fn_name).cloned(),
            integration,
            route,
        }
    }

    // pub fn rm_deployed_components(
    //     &mut self,
    //     route_key: &RouteKey,
    //     fn_name: &String,
    // ) -> AwsLambdaResources {
    //     let route = self.routes.remove(route_key);
    //     let integration = match &route {
    //         None => None,
    //         Some(route) => {
    //             let integration_id = route
    //                 .target
    //                 .as_ref()
    //                 .and_then(|target| target.strip_prefix("integrations/"));
    //             integration_id.and_then(|integration_id| self.integrations.remove(integration_id))
    //         }
    //     };
    //     AwsLambdaResources {
    //         function: self.functions.remove(fn_name),
    //         integration,
    //         route: route.and_then(|r| r.route_id),
    //     }
    // }

    // pub fn collect_deployed_components(
    //     mut self,
    //     project_name: &String,
    // ) -> Vec<AwsDeployedLambdaResources> {
    //     let mut components = Vec::new();
    //     // todo does routes.keys() have to be cloned to use &mut self within the for loop?
    //     for route_key in self.routes.keys().cloned().collect::<Vec<RouteKey>>() {
    //         let fn_name = route_key.to_fn_name(project_name);
    //         components.push(self.rm_deployed_components(&route_key, &fn_name));
    //     }
    //     // todo does integrations.keys() have to be cloned to use &mut self within the for loop?
    //     for integration_id in self
    //         .integrations
    //         .keys()
    //         .cloned()
    //         .collect::<Vec<IntegrationId>>()
    //     {
    //         let integration = self.integrations.remove(&integration_id);
    //         let fn_name = integration
    //             .clone()
    //             .map(|integration| parse_fn_name_from_arn(&integration.integration_uri));
    //         components.push(AwsDeployedLambdaResources {
    //             function: fn_name.and_then(|fn_name| self.functions.remove(&fn_name)),
    //             integration,
    //             route: None,
    //         });
    //     }
    //     components.append(
    //         &mut self
    //             .functions
    //             .into_values()
    //             .map(|function| AwsDeployedLambdaResources {
    //                 function: Some(function),
    //                 integration: None,
    //                 route: None,
    //             })
    //             .collect(),
    //     );
    //     assert!(self.routes.is_empty());
    //     assert!(self.integrations.is_empty());
    //     components
    // }
}
