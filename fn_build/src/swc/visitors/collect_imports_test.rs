use crate::swc::compiler::SwcCompiler;
use crate::swc::visitors::CollectImportsVisitor;
use std::fs;
use swc_ecma_visit::FoldWith;
use temp_dir::TempDir;

fn parse_imports(js: &str) -> Vec<String> {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.child("index.js");
    fs::write(&path, js).unwrap();
    let compiler = SwcCompiler::new();
    let module = compiler.clone().parse_module_from_fs(&path).unwrap();
    let mut visitor = CollectImportsVisitor::new();
    module.fold_with(&mut visitor);
    visitor.result()
}

#[test]
fn test_collect_imports_visitor_collects_top_level_import() {
    let imports = parse_imports("import {getData} from './data.js'\ngetData()");
    assert_eq!(1, imports.len());
    assert_eq!("./data.js", imports.iter().nth(0).unwrap());
}

#[test]
fn test_collect_imports_visitor_collects_import_fn() {
    let imports = parse_imports(
        "async function doWork() {\nconst {getData} = await import('./data.js')\ngetData()\n}",
    );
    assert_eq!(1, imports.len());
    assert_eq!("./data.js", imports.iter().nth(0).unwrap());
}
