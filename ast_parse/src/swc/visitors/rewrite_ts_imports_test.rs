use crate::swc::visitors::RewriteTsImportsVisitor;
use crate::swc::{parse_program_from_fs, parse_program_to_string, SwcCompiler};
use std::fs;
use swc_ecma_visit::FoldWith;
use temp_dir::TempDir;

fn rewrite_imports(js: &str) -> String {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.child("index.ts");
    fs::write(&path, js).unwrap();
    let compiler = Some(SwcCompiler::new());
    let program = parse_program_from_fs(&path, compiler.clone()).unwrap();
    parse_program_to_string(
        program.fold_with(&mut RewriteTsImportsVisitor::new()),
        compiler,
    )
    .unwrap()
}

#[test]
fn test_rewrite_ts_imports_visitor_on_top_level_import() {
    let result = rewrite_imports("import { getData } from './data.ts'\ngetData()");
    assert_eq!(
        result,
        "import { getData } from \"./data.js\";\ngetData();\n"
    );
}

#[test]
fn test_rewrite_ts_imports_visitor_on_import_fn() {
    let result = rewrite_imports(
        "async function doWork() {\nconst {getData} = await import('./data.ts')\ngetData()\n}",
    );
    assert_eq!(result, "async function doWork() {\n    const { getData } = await import(\"./data.js\");\n    getData();\n}\n");
}
