use crate::result::ModuleImport;
use crate::swc::imports::{ImportResolver, NodeImportResolver};
use std::fs;
use std::path::PathBuf;
use temp_dir::TempDir;

#[test]
fn test_node_sibling_relative_import() {
    let project_dir = TempDir::new().unwrap();
    fs::write(project_dir.child("api.js"), "").unwrap();
    fs::write(project_dir.child("data.js"), "").unwrap();

    let resolver = NodeImportResolver {};
    let result = resolver.resolve(project_dir.path(), &PathBuf::from("api.js"), "./data.js");
    match result {
        ModuleImport::RelativeSource(path) => assert_eq!(PathBuf::from("data.js"), path),
        _ => panic!(),
    };
}

#[test]
fn test_node_ancestor_relative_import() {
    let project_dir = TempDir::new().unwrap();
    let api_path = project_dir.child("apis/api.js");
    fs::create_dir(api_path.parent().unwrap()).unwrap();
    fs::write(&api_path, "").unwrap();
    fs::write(project_dir.child("data.js"), "").unwrap();

    let resolver = NodeImportResolver {};
    let result = resolver.resolve(project_dir.path(), &PathBuf::from("api.js"), "./data.js");
    match result {
        ModuleImport::RelativeSource(path) => assert_eq!(PathBuf::from("data.js"), path),
        _ => panic!(),
    };
}
