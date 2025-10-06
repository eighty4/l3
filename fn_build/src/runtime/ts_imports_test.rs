use crate::runtime::node::NodeConfig;
use crate::runtime::ts_imports::TypeScriptImportResolver;
use crate::runtime::ImportResolver;
use crate::ModuleImport;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

struct TestFallthrough {
    unresolved: Arc<Mutex<Vec<(PathBuf, String)>>>,
}

impl ImportResolver for TestFallthrough {
    fn resolve(&self, _project_dir: &Path, from: &Path, import: &str) -> ModuleImport {
        self.unresolved
            .lock()
            .unwrap()
            .push((from.to_path_buf(), String::from(import)));
        ModuleImport::Unknown(String::from(import))
    }
}

struct TsImportFixture {
    project_dir: PathBuf,
    import_resolver: TypeScriptImportResolver,
    unresolved: Arc<Mutex<Vec<(PathBuf, String)>>>,
}

fn create_import_resolver_for_fixture(fixture_dir: &str) -> TsImportFixture {
    let project_dir = PathBuf::from(fixture_dir);
    let node_config = NodeConfig::read_configs(&project_dir).unwrap();
    let unresolved: Arc<Mutex<Vec<(PathBuf, String)>>> = Default::default();
    let import_resolver = TypeScriptImportResolver::new(
        node_config.ts.unwrap().clone(),
        Box::new(TestFallthrough {
            unresolved: unresolved.clone(),
        }),
    );
    TsImportFixture {
        project_dir,
        import_resolver,
        unresolved,
    }
}

#[test]
fn test_ts_import_resolver_resolves_relative_source() {
    let test = create_import_resolver_for_fixture("fixtures/node/ts/import_uses_js");
    assert_eq!(
        test.import_resolver.resolve(
            &test.project_dir,
            &PathBuf::from("routes/data/lambda.ts"),
            "../../lib/data.js",
        ),
        ModuleImport::RelativeSource(PathBuf::from("lib/data.ts"))
    );
    assert!(test.unresolved.lock().unwrap().is_empty());
}

#[test]
fn test_ts_import_resolver_delegates_to_runtime_resolver() {
    let test = create_import_resolver_for_fixture("fixtures/node/ts/import_uses_js");
    assert_eq!(
        test.import_resolver.resolve(
            &test.project_dir,
            &PathBuf::from("routes/data/lambda.ts"),
            "../../lib/mongodb.js",
        ),
        ModuleImport::Unknown(String::from("../../lib/mongodb.js"))
    );
    let unresolved = test.unresolved.lock().unwrap();
    assert!(!unresolved.is_empty());
    let (from, import) = unresolved.first().unwrap();
    assert_eq!(from, &PathBuf::from("routes/data/lambda.ts"));
    assert_eq!(import, &String::from("../../lib/mongodb.js"));
}
