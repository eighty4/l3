use aws_sdk_apigatewayv2::types::{Integration, Route};
use aws_sdk_lambda::types::FunctionConfiguration;

use crate::aws::state::DeployedProjectState;
use crate::lambda::{HttpMethod, RouteKey};

#[test]
fn test_deployed_state_resolves_lambda_components_by_route_key() {
    let state = DeployedProjectState::new(
        &"this_project".to_string(),
        vec![
            FunctionConfiguration::builder()
                .function_arn("not the function arn")
                .function_name("l3-other_project-not the function name")
                .build(),
            FunctionConfiguration::builder()
                .function_arn("arn:aws:lambda:region:account:l3-this_project-some-function")
                .function_name("l3-this_project-some-function")
                .build(),
        ],
        vec![
            Integration::builder()
                .integration_id("not the integration id")
                .integration_uri("not the function arn")
                .build(),
            Integration::builder()
                .integration_id("integration id")
                .integration_uri("arn:aws:lambda:region:account:l3-this_project-some-function")
                .build(),
        ],
        vec![
            Route::builder()
                .route_id("not the route id")
                .route_key("PATCH /not/this/route")
                .target("")
                .build(),
            Route::builder()
                .route_id("route id")
                .route_key("GET /BINGO")
                .target("integrations/integration id")
                .build(),
        ],
    );
    let components = state.get_deployed_components(
        &String::from("l3-this_project-some-function"),
        &RouteKey::new(HttpMethod::Get, "BINGO".to_string()),
    );
    assert_eq!(components.route.unwrap(), "route id");
    assert_eq!(components.integration.unwrap(), "integration id");
    assert_eq!(
        components.function.unwrap(),
        "arn:aws:lambda:region:account:l3-this_project-some-function"
    );
}
