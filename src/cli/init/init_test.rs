use crate::cli::init::{init_project, InitOptions};
use crate::code::source::Language::{JavaScript, Python, TypeScript};
use std::fs;
use std::path::Path;
use temp_dir::TempDir;

fn collect_gitignore_entries(p: &Path) -> Vec<String> {
    fs::read_to_string(p)
        .unwrap()
        .lines()
        .map(|s| s.to_string())
        .collect()
}

#[test]
fn test_init_project_creates_js_project() {
    let project_dir = TempDir::new().unwrap();
    init_project(InitOptions {
        language: Some(JavaScript),
        project_dir: project_dir.path().to_path_buf(),
        project_name: Some("my_project".to_string()),
    })
    .unwrap();
    assert!(project_dir.path().join("routes/lambda.js").is_file());
    let gitignore_entries = collect_gitignore_entries(&project_dir.path().join(".gitignore"));
    assert!(gitignore_entries.contains(&".l3".to_string()));
    assert!(gitignore_entries.contains(&"node_modules".to_string()));
    assert_eq!(gitignore_entries.len(), 2);
}

#[test]
fn test_init_project_creates_py_project() {
    let project_dir = TempDir::new().unwrap();
    init_project(InitOptions {
        language: Some(Python),
        project_dir: project_dir.path().to_path_buf(),
        project_name: Some("my_project".to_string()),
    })
    .unwrap();
    assert!(project_dir.path().join("routes/lambda.py").is_file());
    let gitignore_entries = collect_gitignore_entries(&project_dir.path().join(".gitignore"));
    assert!(gitignore_entries.contains(&".l3".to_string()));
    assert_eq!(gitignore_entries.len(), 1);
}

#[test]
fn test_init_project_creates_ts_project() {
    let project_dir = TempDir::new().unwrap();
    init_project(InitOptions {
        language: Some(TypeScript),
        project_dir: project_dir.path().to_path_buf(),
        project_name: Some("my_project".to_string()),
    })
    .unwrap();
    assert!(project_dir.path().join("routes/lambda.ts").is_file());
    let gitignore_entries = collect_gitignore_entries(&project_dir.path().join(".gitignore"));
    assert!(gitignore_entries.contains(&".l3".to_string()));
    assert!(gitignore_entries.contains(&"node_modules".to_string()));
    assert_eq!(gitignore_entries.len(), 2);
}

#[test]
fn test_init_project_updates_existing_gitignore_without_dupes_with_eof_newline() {
    let project_dir = TempDir::new().unwrap();
    let gitignore = project_dir.path().join(".gitignore");
    fs::write(&gitignore, ".idea\n.l3\n").unwrap();
    init_project(InitOptions {
        language: Some(JavaScript),
        project_dir: project_dir.path().to_path_buf(),
        project_name: Some("my_project".to_string()),
    })
    .unwrap();
    assert_eq!(
        ".idea\n.l3\nnode_modules",
        fs::read_to_string(gitignore).unwrap()
    );
}

#[test]
fn test_init_project_updates_existing_gitignore_without_dupes_without_eof_newline() {
    let project_dir = TempDir::new().unwrap();
    let gitignore = project_dir.path().join(".gitignore");
    fs::write(&gitignore, ".idea\n.l3").unwrap();
    init_project(InitOptions {
        language: Some(JavaScript),
        project_dir: project_dir.path().to_path_buf(),
        project_name: Some("my_project".to_string()),
    })
    .unwrap();
    assert_eq!(
        ".idea\n.l3\nnode_modules",
        fs::read_to_string(gitignore).unwrap()
    );
}

#[test]
fn test_init_project_errors_if_config_file_exists() {
    let project_dir = TempDir::new().unwrap();
    fs::write(project_dir.path().join("l3.yml"), "").unwrap();
    assert!(init_project(InitOptions {
        language: Some(JavaScript),
        project_dir: project_dir.path().to_path_buf(),
        project_name: Some("my_project".to_string()),
    })
    .is_err());
}

#[test]
fn test_init_project_errors_if_data_dir_exists() {
    let project_dir = TempDir::new().unwrap();
    fs::create_dir(project_dir.path().join(".l3")).unwrap();
    assert!(init_project(InitOptions {
        language: Some(JavaScript),
        project_dir: project_dir.path().to_path_buf(),
        project_name: Some("my_project".to_string()),
    })
    .is_err());
}

#[test]
fn test_init_project_errors_if_routes_dir_exists() {
    let project_dir = TempDir::new().unwrap();
    fs::create_dir(project_dir.path().join("routes")).unwrap();
    assert!(init_project(InitOptions {
        language: Some(JavaScript),
        project_dir: project_dir.path().to_path_buf(),
        project_name: Some("my_project".to_string()),
    })
    .is_err());
}
