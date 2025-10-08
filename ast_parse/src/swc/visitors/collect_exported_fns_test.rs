use crate::swc::{parse_program_from_fs, visitors::CollectExportedFnsVisitor};
use std::fs;
use swc_ecma_visit::FoldWith;
use temp_dir::TempDir;

fn parse_exports(js: &str) -> Vec<String> {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.child("index.js");
    fs::write(&path, js).unwrap();
    let module = parse_program_from_fs(&path, None).unwrap();
    let mut visitor = CollectExportedFnsVisitor::new();
    module.fold_with(&mut visitor);
    visitor.result()
}

#[test]
fn test_collect_exports_visitor_collects_export_function() {
    let exports = parse_exports("export function put() {}");
    assert_eq!(1, exports.len());
    assert_eq!("put", exports.first().unwrap());
}

#[test]
fn test_collect_exports_visitor_collects_export_const_arrow_lambda() {
    let exports = parse_exports("export const patch = () => {}");
    assert_eq!(1, exports.len());
    assert_eq!("patch", exports.first().unwrap());
}
