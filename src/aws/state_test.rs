use aws_sdk_apigatewayv2::types::{Integration, Route};
use aws_sdk_lambda::types::{EnvironmentResponse, FunctionConfiguration};
use std::collections::HashMap;

use crate::aws::state::DeployedProjectState;
use crate::code::env::EnvVarSources;
use crate::lambda::{HttpMethod, LambdaFn, RouteKey};
use crate::testing::{ProjectTest, TestSource};

#[tokio::test]
async fn test_deployed_state_resolves_lambda_components_by_route_key() {
    let project_test = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("this_project")
        .with_source(
            TestSource::with_path("routes/some/function/lambda.js")
                .content("export function GET(){}"),
        )
        .build();
    let route_key = RouteKey::new(HttpMethod::Get, "some/function".to_string());
    let lambda_fn = LambdaFn::new(
        EnvVarSources::new(&project_test.project_dir, &route_key).unwrap(),
        "GET".to_string(),
        project_test.source_path("routes/some/function/lambda.js"),
        project_test.project_deets.clone(),
        route_key,
    );
    let state = DeployedProjectState::new(
        &"this_project".to_string(),
        vec![
            FunctionConfiguration::builder()
                .function_arn("not the function arn")
                .function_name("l3-other_project-not the function name")
                .build(),
            FunctionConfiguration::builder()
                .function_arn("arn:aws:lambda:region:account:l3-this_project-some-function-get")
                .environment(
                    EnvironmentResponse::builder()
                        .set_variables(Some(HashMap::from([(
                            "KEY".to_string(),
                            "VAL".to_string(),
                        )])))
                        .build(),
                )
                .function_name("l3-this_project-some-function-get")
                .build(),
        ],
        vec![
            Integration::builder()
                .integration_id("not the integration id")
                .integration_uri("not the function arn")
                .build(),
            Integration::builder()
                .integration_id("integration id")
                .integration_uri("arn:aws:lambda:region:account:l3-this_project-some-function-get")
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
                .route_key("GET /some/function")
                .target("integrations/integration id")
                .build(),
        ],
    );
    let components = state.get_deployed_components(&lambda_fn);
    assert_eq!(components.route.unwrap(), "route id");
    let (integration_id, integration_uri) = components.integration.unwrap();
    assert_eq!(integration_id, "integration id");
    assert_eq!(
        integration_uri,
        "arn:aws:lambda:region:account:l3-this_project-some-function-get"
    );
    assert_eq!(
        components.function_arn.unwrap(),
        "arn:aws:lambda:region:account:l3-this_project-some-function-get"
    );
    assert_eq!(
        components.function_env.get(&"KEY".to_string()).unwrap(),
        &"VAL".to_string()
    );
    assert_eq!(
        components.function_name.unwrap(),
        "l3-this_project-some-function-get"
    );
}
