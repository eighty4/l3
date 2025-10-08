use std::fs;

use temp_dir::TempDir;

use crate::collect_exported_fns;

#[test]
fn test_collect_exported_fns_for_export_function() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.child("lambda.js");
    fs::write(&path, "export function get() {}").unwrap();
    assert_eq!(collect_exported_fns(&path).unwrap(), vec!["get"]);
}

#[test]
fn test_collect_exported_fns_for_export_const_arrow_lambda() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.child("lambda.js");
    fs::write(&path, "export const get = () => {}").unwrap();
    assert_eq!(collect_exported_fns(&path).unwrap(), vec!["get"]);
}
