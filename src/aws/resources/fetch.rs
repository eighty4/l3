use crate::aws::clients::AwsClients;
use crate::aws::AwsApiGateway;
use anyhow::anyhow;
use aws_sdk_apigatewayv2::config::http::HttpResponse;
use aws_sdk_apigatewayv2::error::SdkError;
use aws_sdk_apigatewayv2::operation::get_integrations::GetIntegrationsError;
use aws_sdk_apigatewayv2::operation::get_routes::GetRoutesError;
use aws_sdk_apigatewayv2::types::{Integration, Route};
use aws_sdk_lambda::operation::list_functions::ListFunctionsError;
use aws_sdk_lambda::types::FunctionConfiguration;
use std::sync::Arc;
use tokio::task::JoinSet;

const MAX_ITEMS_PER_PAGE: i32 = 20;

enum FetchResult {
    Functions(Vec<FunctionConfiguration>),
    Integrations(Vec<Integration>),
    Routes(Vec<Route>),
}

pub type FetchedData = (Vec<FunctionConfiguration>, Vec<Integration>, Vec<Route>);

pub async fn fetch_project_state(
    sdk_clients: &Arc<AwsClients>,
    api_gateway: &Arc<AwsApiGateway>,
) -> Result<FetchedData, anyhow::Error> {
    let mut join_set: JoinSet<Result<FetchResult, anyhow::Error>> = JoinSet::new();
    join_set.spawn(collect_functions(sdk_clients.clone()));
    join_set.spawn(collect_integrations(
        sdk_clients.clone(),
        api_gateway.clone(),
    ));
    join_set.spawn(collect_routes(sdk_clients.clone(), api_gateway.clone()));
    let mut functions: Option<Vec<FunctionConfiguration>> = None;
    let mut integrations: Option<Vec<Integration>> = None;
    let mut routes: Option<Vec<Route>> = None;
    loop {
        match join_set.join_next().await {
            None => break,
            Some(result) => match result {
                Ok(fetched) => match fetched {
                    Ok(FetchResult::Functions(fetched_functions)) => {
                        functions = Some(fetched_functions)
                    }
                    Ok(FetchResult::Integrations(fetched_integrations)) => {
                        integrations = Some(fetched_integrations)
                    }
                    Ok(FetchResult::Routes(fetched_routes)) => routes = Some(fetched_routes),
                    Err(err) => panic!("{}", err),
                },
                Err(err) => panic!("{}", err),
            },
        }
    }
    Ok((
        functions.unwrap_or_default(),
        integrations.unwrap_or_default(),
        routes.unwrap_or_default(),
    ))
}

async fn collect_functions(sdk_clients: Arc<AwsClients>) -> Result<FetchResult, anyhow::Error> {
    let mut functions: Vec<FunctionConfiguration> = Vec::new();
    let mut next_marker: Option<String> = None;
    loop {
        let mut builder = sdk_clients
            .lambda
            .list_functions()
            .max_items(MAX_ITEMS_PER_PAGE);
        if next_marker.is_some() {
            builder = builder.marker(next_marker.unwrap())
        }
        let response = builder.send().await.map_err(map_list_functions_error)?;
        functions.append(&mut response.functions.unwrap());
        if response.next_marker.is_some() {
            next_marker = response.next_marker;
        } else {
            break;
        }
    }
    Ok(FetchResult::Functions(functions))
}

async fn collect_integrations(
    sdk_clients: Arc<AwsClients>,
    api_gateway: Arc<AwsApiGateway>,
) -> Result<FetchResult, anyhow::Error> {
    let mut integrations: Vec<Integration> = Vec::new();
    let mut next_token: Option<String> = None;
    loop {
        let mut builder = sdk_clients
            .api_gateway
            .get_integrations()
            .api_id(api_gateway.id.as_str())
            .max_results(MAX_ITEMS_PER_PAGE.to_string());
        if next_token.is_some() {
            builder = builder.next_token(next_token.unwrap())
        }
        let response = builder.send().await.map_err(map_get_integrations_error)?;
        integrations.append(&mut response.items.unwrap());
        if response.next_token.is_some() {
            next_token = response.next_token;
        } else {
            break;
        }
    }
    Ok(FetchResult::Integrations(integrations))
}

async fn collect_routes(
    sdk_clients: Arc<AwsClients>,
    api_gateway: Arc<AwsApiGateway>,
) -> Result<FetchResult, anyhow::Error> {
    let mut routes: Vec<Route> = Vec::new();
    let mut next_token: Option<String> = None;
    loop {
        let mut builder = sdk_clients
            .api_gateway
            .get_routes()
            .api_id(api_gateway.id.as_str())
            .max_results(MAX_ITEMS_PER_PAGE.to_string());
        if next_token.is_some() {
            builder = builder.next_token(next_token.unwrap())
        }
        let response = builder.send().await.map_err(map_get_routes_error)?;
        routes.append(&mut response.items.unwrap());
        if response.next_token.is_some() {
            next_token = response.next_token;
        } else {
            break;
        }
    }
    Ok(FetchResult::Routes(routes))
}

fn map_list_functions_error(e: SdkError<ListFunctionsError, HttpResponse>) -> anyhow::Error {
    anyhow!("{}", e.map_service_error(|e| e.to_string()))
}

fn map_get_integrations_error(e: SdkError<GetIntegrationsError, HttpResponse>) -> anyhow::Error {
    anyhow!("{}", e.map_service_error(|e| e.to_string()))
}

fn map_get_routes_error(e: SdkError<GetRoutesError, HttpResponse>) -> anyhow::Error {
    anyhow!("{}", e.map_service_error(|e| e.to_string()))
}
