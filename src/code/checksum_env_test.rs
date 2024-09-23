use crate::code::checksum::ChecksumTree;
use crate::code::env::EnvVarSources;
use crate::lambda::{HttpMethod, RouteKey};
use crate::testing::project::ProjectTest;
use crate::testing::source::TestSource;
use std::collections::HashMap;
use std::fs;

enum ChecksumState {
    Absent,
    Clean,
    Dirty,
}

async fn test_checksum_tree_do_env_checksums_match(
    method_env: Option<HashMap<String, String>>,
    path_env: Option<HashMap<String, String>>,
    method_checksum: ChecksumState,
    path_checksum: ChecksumState,
    deployed_env: Option<HashMap<String, String>>,
    do_env_checksums_match: bool,
) {
    let route_key = RouteKey::new(HttpMethod::Get, "data".to_string());
    let mut project_test_builder = ProjectTest::builder()
        .api_id("API_ID")
        .project_name("PROJECT_NAME");
    if let Some(method_env) = &method_env {
        let method_env_builder = TestSource::method_env_var(route_key.clone())
            .content(env_map_to_string(method_env).as_ref());
        project_test_builder = project_test_builder.with_source(match method_checksum {
            ChecksumState::Absent => method_env_builder.without_checksum(),
            ChecksumState::Clean => method_env_builder.with_clean_checksum(),
            ChecksumState::Dirty => method_env_builder.with_dirty_checksum(),
        });
    }
    if let Some(path_env) = &path_env {
        let path_env_builder = TestSource::path_env_var(route_key.clone())
            .content(env_map_to_string(path_env).as_ref());
        project_test_builder = project_test_builder.with_source(match path_checksum {
            ChecksumState::Absent => path_env_builder.without_checksum(),
            ChecksumState::Clean => path_env_builder.with_clean_checksum(),
            ChecksumState::Dirty => path_env_builder.with_dirty_checksum(),
        });
    }
    let project_test = project_test_builder.build();

    if let None = method_env {
        match method_checksum {
            ChecksumState::Clean | ChecksumState::Dirty => {
                let checksum_path = &project_test
                    .lambda_checksum_path(&route_key)
                    .join(&project_test.lambda_route_dir(&route_key))
                    .join("lambda.get.env");
                fs::create_dir_all(&checksum_path).unwrap();
                fs::write(checksum_path, "lorem ipsum").unwrap();
            }
            _ => {}
        }
    }
    if let None = path_env {
        match path_checksum {
            ChecksumState::Clean | ChecksumState::Dirty => {
                let checksum_path = &project_test
                    .lambda_checksum_path(&route_key)
                    .join(&project_test.lambda_route_dir(&route_key))
                    .join("lambda.env");
                fs::create_dir_all(&checksum_path).unwrap();
                fs::write(checksum_path, "lorem ipsum").unwrap();
            }
            _ => {}
        }
    }

    let checksum_tree = ChecksumTree::new(
        project_test.project_dir.clone(),
        &project_test.api_id,
        &route_key.to_fn_name(&project_test.project_name),
    )
    .await
    .unwrap();
    let env_var_sources = EnvVarSources::new(&project_test.project_dir, &route_key).unwrap();

    assert_eq!(
        do_env_checksums_match,
        checksum_tree
            .do_env_checksums_match(&deployed_env, &env_var_sources)
            .unwrap()
    );
}

fn env_map_to_string(env_map: &HashMap<String, String>) -> String {
    env_map.iter().map(|(k, v)| format!("{k}={v}\n")).collect()
}

#[tokio::test]
async fn test_with_clean_method_checksum() {
    for env in [None, Some(HashMap::new())] {
        test_checksum_tree_do_env_checksums_match(
            Some(HashMap::from([("KEY".to_string(), "VAL".to_string())])),
            None,
            ChecksumState::Clean,
            ChecksumState::Absent,
            env,
            true,
        )
        .await;
    }
}

#[tokio::test]
async fn test_with_clean_path_checksum() {
    for env in [None, Some(HashMap::new())] {
        test_checksum_tree_do_env_checksums_match(
            None,
            Some(HashMap::from([("KEY".to_string(), "VAL".to_string())])),
            ChecksumState::Absent,
            ChecksumState::Clean,
            env,
            true,
        )
        .await;
    }
}

#[tokio::test]
async fn test_with_dirty_method_checksum() {
    for env in [None, Some(HashMap::new())] {
        test_checksum_tree_do_env_checksums_match(
            Some(HashMap::from([("KEY".to_string(), "VAL".to_string())])),
            None,
            ChecksumState::Dirty,
            ChecksumState::Absent,
            env,
            false,
        )
        .await;
    }
}

#[tokio::test]
async fn test_with_dirty_path_checksum() {
    for env in [None, Some(HashMap::new())] {
        test_checksum_tree_do_env_checksums_match(
            None,
            Some(HashMap::from([("KEY".to_string(), "VAL".to_string())])),
            ChecksumState::Absent,
            ChecksumState::Dirty,
            env,
            false,
        )
        .await;
    }
}

#[tokio::test]
async fn test_without_method_checksum() {
    for env in [None, Some(HashMap::new())] {
        test_checksum_tree_do_env_checksums_match(
            Some(HashMap::from([("KEY".to_string(), "VAL".to_string())])),
            None,
            ChecksumState::Absent,
            ChecksumState::Absent,
            env,
            false,
        )
        .await;
    }
}

#[tokio::test]
async fn test_without_path_checksum() {
    for env in [None, Some(HashMap::new())] {
        test_checksum_tree_do_env_checksums_match(
            None,
            Some(HashMap::from([("KEY".to_string(), "VAL".to_string())])),
            ChecksumState::Absent,
            ChecksumState::Absent,
            env,
            false,
        )
        .await;
    }
}

#[tokio::test]
async fn test_without_env_or_checksum_files_with_deployed_env() {
    test_checksum_tree_do_env_checksums_match(
        None,
        None,
        ChecksumState::Absent,
        ChecksumState::Absent,
        Some(HashMap::from([("KEY".to_string(), "VAL".to_string())])),
        false,
    )
    .await;
}
