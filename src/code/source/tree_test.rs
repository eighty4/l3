use crate::code::source::tree::SourceTree;
use crate::code::source::Language;
use crate::lambda::{HttpMethod, RouteKey};
use crate::testing::{ProjectTest, TestSource};
use tokio::sync::mpsc::unbounded_channel;

#[tokio::test]
async fn test_sources_api_refresh_routes() {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::http_fn(Language::TypeScript, route_key.clone())
                .content("import {getData} from '../lib/data'\nexport const GET = () => {}"),
        )
        .with_source(
            TestSource::with_path("lib/data.js").content("export const getData = () => 42"),
        )
        .build();

    let (tx, mut rx) = unbounded_channel();
    let (source_tree, sources_api) = SourceTree::new(tx, project_test.project_deets.clone());
    sources_api.refresh_routes().await.unwrap();

    let lambda_fns = source_tree.lock().unwrap().lambda_fns();
    assert_eq!(1, lambda_fns.len());
    let lambda_fn_by_route_key = source_tree
        .lock()
        .unwrap()
        .lambda_fn_by_route_key(&route_key);
    assert!(lambda_fn_by_route_key.is_some());
}

// todo test compilation error sends LambdaNotification
// todo test duplicate HTTP handler functions sends LambdaNotification
// todo test routes source file without HTTP handle function sends LambdaNotification

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
