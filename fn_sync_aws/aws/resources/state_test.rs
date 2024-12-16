use aws_sdk_apigatewayv2::types::{Integration, Route};
use aws_sdk_lambda::types::{EnvironmentResponse, FunctionConfiguration, Runtime};
use std::collections::HashMap;

use crate::aws::resources::state::DeployedProjectState;
use crate::lambda::{HttpMethod, LambdaFn, RouteKey};
use crate::testing::project::ProjectTest;
use crate::testing::source::TestSource;

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
        "GET".to_string(),
        project_test.source_path("routes/some/function/lambda.js"),
        project_test.project.clone(),
        route_key,
    );
    let state = DeployedProjectState::new(
        &"this_project".to_string(),
        vec![
            FunctionConfiguration::builder()
                .function_arn("not the function arn")
                .function_name("l3-other_project-not the function name")
                .handler("")
                .role("")
                .runtime(Runtime::Nodejs20x)
                .build(),
            FunctionConfiguration::builder()
                .environment(
                    EnvironmentResponse::builder()
                        .set_variables(Some(HashMap::from([(
                            "KEY".to_string(),
                            "VAL".to_string(),
                        )])))
                        .build(),
                )
                .function_arn("arn:aws:lambda:region:account:l3-this_project-some-function-get")
                .function_name("l3-this_project-some-function-get")
                .handler("")
                .role("")
                .runtime(Runtime::Python312)
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
    assert_eq!(components.route.unwrap().id, "route id");
    let integration = components.integration.unwrap();
    assert_eq!(integration.id, "integration id");
    assert_eq!(
        integration.integration_uri,
        "arn:aws:lambda:region:account:l3-this_project-some-function-get"
    );
    let function = components.function.unwrap();
    assert_eq!(
        &function.arn,
        "arn:aws:lambda:region:account:l3-this_project-some-function-get"
    );
    assert_eq!(
        &function
            .env
            .as_ref()
            .and_then(|env| env.get(&"KEY".to_string()))
            .cloned()
            .unwrap(),
        &"VAL".to_string()
    );
    assert_eq!(&function.name, "l3-this_project-some-function-get");
}
