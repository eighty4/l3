use std::fs;
use temp_dir::TempDir;

use crate::collect_exported_fns;

#[test]
fn test_collect_exported_fns() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.child("lambda.py");
    fs::write(&path, "def get():\n    return 'data'\n").unwrap();
    assert_eq!(collect_exported_fns(&path).unwrap(), vec!["get"]);
}
