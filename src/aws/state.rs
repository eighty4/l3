use std::collections::{HashMap, HashSet};

use aws_sdk_apigatewayv2::types::{ConnectionType, Integration, IntegrationType, Route};

use crate::aws::clients::AwsClients;
use crate::aws::lambda::FunctionName;
use crate::lambda::{LambdaFn, RouteKey};

pub type IntegrationId = String;

pub struct DeployedLambdaComponents {
    pub function: Option<FunctionName>,
    pub integration: Option<IntegrationId>,
    pub route: Option<RouteKey>,
}

pub struct DeployedProjectState {
    pub functions: HashSet<FunctionName>,
    pub integrations: Vec<Integration>,
    pub routes: HashMap<RouteKey, Route>,
}

impl DeployedProjectState {
    pub async fn fetch_state(
        sdk_clients: &AwsClients,
        api_id: &String,
    ) -> Result<Self, anyhow::Error> {
        let list_functions_output = sdk_clients.lambda.list_functions().send().await?;

        let mut functions = HashSet::new();
        if let Some(fetched_functions) = list_functions_output.functions {
            for function in fetched_functions {
                functions.insert(function.function_name.unwrap());
            }
        }

        let get_routes_output = sdk_clients
            .api_gateway
            .get_routes()
            .api_id(api_id)
            .send()
            .await?;

        let mut routes = HashMap::new();
        if let Some(fetched_routes) = get_routes_output.items {
            for route in fetched_routes {
                let route_key = RouteKey::try_from(route.route_key.clone().unwrap())?;
                routes.insert(route_key, route);
            }
        }

        let get_integrations_output = sdk_clients
            .api_gateway
            .get_integrations()
            .api_id(api_id)
            .send()
            .await?;

        let mut integrations = Vec::new();
        if let Some(fetched_integrations) = get_integrations_output.items {
            for integration in fetched_integrations {
                if is_valid_integration(&integration) {
                    integrations.push(integration);
                }
            }
        }

        Ok(Self {
            functions,
            integrations,
            routes,
        })
    }

    pub fn get_deployed_components(&self, lambda_fn: &LambdaFn) -> DeployedLambdaComponents {
        DeployedLambdaComponents {
            function: self.functions.get(&lambda_fn.fn_name).cloned(),
            integration: self.get_integration_id_by_fn_name(&lambda_fn.fn_name),
            route: self.get_route_by_route_key(&lambda_fn.route_key),
        }
    }

    fn get_integration_id_by_fn_name(&self, fn_name: &FunctionName) -> Option<IntegrationId> {
        for integration in self.integrations.iter() {
            let arn = integration.integration_uri.clone().unwrap();
            if arn.starts_with("arn:aws:lambda") && arn.ends_with(fn_name.as_str()) {
                return Some(integration.integration_id.clone().unwrap());
            }
        }
        None
    }

    fn get_route_by_route_key(&self, route_key: &RouteKey) -> Option<RouteKey> {
        self.routes
            .get(route_key)
            .map(|r| RouteKey::try_from(r.route_key.clone().unwrap()).unwrap())
    }
}

fn is_valid_integration(integration: &Integration) -> bool {
    let mut valid = true;
    if integration.connection_type.is_none() || integration.integration_type.is_none() {
        valid = false;
    } else if let Some(ct) = &integration.connection_type {
        valid = valid && matches!(ct, ConnectionType::Internet);
        if let Some(it) = &integration.integration_type {
            valid = valid && matches!(it, IntegrationType::AwsProxy);
        }
    }
    valid
}
