use crate::code::source::tree::SourceTree;
use crate::code::source::Language;
use crate::lambda::{HttpMethod, RouteKey};
use crate::testing::project::ProjectTest;
use crate::testing::source::TestSource;
use std::path::PathBuf;

#[tokio::test]
async fn test_sources_api_refresh_routes() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::http_fn(Language::JavaScript, route_key.clone())
                .content("import {getData} from '../../lib/data.js'\nexport const GET = () => {}"),
        )
        .with_source(
            TestSource::with_path("lib/data.js").content("export const getData = () => 42"),
        )
        .build();

    let (source_tree, sources_api) = SourceTree::new(project_test.project.clone());
    sources_api.refresh_routes().await.unwrap();

    let mut source_tree = source_tree.lock().unwrap();
    let lambda_fns = source_tree.lambda_fns();
    assert_eq!(1, lambda_fns.len());
    assert!(source_tree.lambda_fn_by_route_key(&route_key).is_some());
    assert!(source_tree
        .get_source_file(&PathBuf::from("routes/data/lambda.js"))
        .is_some());
    assert!(source_tree
        .get_source_file(&PathBuf::from("lib/data.js"))
        .is_some());
}

// todo test compilation error sends LambdaNotification
// todo test duplicate HTTP handler functions sends LambdaNotification
// todo test routes source file without HTTP handle function sends LambdaNotification
// todo test using a TypeScript path alias for a `$lib/data.js` style import
// todo test resolving extension to .js, .mjs or .ts with an extensionless `lib/data` style import

// let mut timeout = interval_at(
//     Instant::now() + Duration::from_secs(10),
//     Duration::from_secs(10),
// );
// select! {
//     notification_opt = rx.recv() => {
//         match notification_opt.unwrap() {
//             LambdaNotification::SourcesEvent(SourcesEvent::RoutesRefreshed) => {
//             }
//             _ => panic!(),
//         }
//     }
//     _ = timeout.tick() => panic!()
// }
