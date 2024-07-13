use std::path::PathBuf;

use crate::code::project::javascript::{
    JavaScriptDeets, NodeConditionalImport, NodeConditionalImportKind,
};

#[test]
fn test_javascript_deets_map_subpath_import_without_match() {
    let js = JavaScriptDeets::default();
    assert!(js.map_subpath_import("#db-dep").is_none());
}

#[test]
fn test_javascript_deets_map_subpath_import_matches_explicit_import() {
    let mut js = JavaScriptDeets::default();
    js.subpath_imports.insert(
        "#db-dep".to_string(),
        vec![NodeConditionalImport {
            kind: NodeConditionalImportKind::Default,
            path: PathBuf::from("./db_dep.js"),
        }],
    );
    match js.map_subpath_import("#db-dep") {
        None => panic!(),
        Some(p) => assert_eq!(p, PathBuf::from("./db_dep.js")),
    }
}

#[test]
fn test_javascript_deets_map_subpath_import_matches_wildcard_to_explicit_path() {
    let mut js = JavaScriptDeets::default();
    js.subpath_imports.insert(
        "#db-dep/*.js".to_string(),
        vec![NodeConditionalImport {
            kind: NodeConditionalImportKind::Default,
            path: PathBuf::from("./lib/db/code.js"),
        }],
    );
    match js.map_subpath_import("#db-dep/data.js") {
        None => panic!(),
        Some(p) => assert_eq!(p, PathBuf::from("./lib/db/code.js")),
    }
}

#[test]
fn test_javascript_deets_map_subpath_import_matches_wildcard_mapping() {
    let mut js = JavaScriptDeets::default();
    js.subpath_imports.insert(
        "#db-dep/*.js".to_string(),
        vec![NodeConditionalImport {
            kind: NodeConditionalImportKind::Default,
            path: PathBuf::from("./lib/db/*.js"),
        }],
    );
    match js.map_subpath_import("#db-dep/data.js") {
        None => panic!(),
        Some(p) => assert_eq!(p, PathBuf::from("./lib/db/data.js")),
    }
}
