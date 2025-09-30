use crate::runtime::node::imports::resolver::NodeImportResolver;
use crate::runtime::node::NodeConfig;
use crate::runtime::ImportResolver;
use crate::ModuleImport;
use std::path::PathBuf;
use std::sync::Arc;

fn create_import_resolver_for_fixture(fixture_dir: &str) -> (PathBuf, NodeImportResolver) {
    let project_dir = PathBuf::from(fixture_dir);
    let import_resolver =
        NodeImportResolver::new(Arc::new(NodeConfig::read_configs(&project_dir).unwrap()));
    (project_dir, import_resolver)
}

#[test]
fn test_node_import_resolver_resolves_relative_source() {
    let (project_dir, import_resolver) =
        create_import_resolver_for_fixture("fixtures/node/js/relative_import");
    assert_eq!(
        import_resolver.resolve(
            &project_dir,
            &PathBuf::from("routes/data/lambda.js"),
            "../../lib/data.js",
        ),
        ModuleImport::RelativeSource(PathBuf::from("lib/data.js"))
    );
}

mod subpath_imports {
    use super::*;

    mod from_explicit {
        use super::*;

        #[test]
        fn test_resolves_from_explicit_to_relative_source() {
            let (project_dir, import_resolver) = create_import_resolver_for_fixture(
                "fixtures/node/js/subpath_import/from_explicit/to_relative_source",
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data.js",
                ),
                ModuleImport::RelativeSource(PathBuf::from("lib/redis.js"))
            );
        }

        #[test]
        fn test_resolves_from_explicit_to_package_dependency() {
            let (project_dir, import_resolver) = create_import_resolver_for_fixture(
                "fixtures/node/js/subpath_import/from_explicit/to_package_dependency",
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data.js",
                ),
                ModuleImport::PackageDependency {
                    package: "data-dep".to_string(),
                    subpath: None
                }
            );
        }
    }

    mod from_asterisk {
        use super::*;

        #[test]
        fn test_resolves_from_asterisk_to_asterisk() {
            let (project_dir, import_resolver) = create_import_resolver_for_fixture(
                "fixtures/node/js/subpath_import/from_asterisk/to_asterisk",
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data/raw.js",
                ),
                ModuleImport::RelativeSource(PathBuf::from("data/raw.js"))
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data/abstraction/orm.js",
                ),
                ModuleImport::RelativeSource(PathBuf::from("data/abstraction/orm.js"))
            );
        }

        #[test]
        fn test_resolves_from_asterisk_to_asterisk_extension() {
            let (project_dir, import_resolver) = create_import_resolver_for_fixture(
                "fixtures/node/js/subpath_import/from_asterisk/to_asterisk_extension",
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data/raw",
                ),
                ModuleImport::RelativeSource(PathBuf::from("data/raw.js"))
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data/abstraction/orm",
                ),
                ModuleImport::RelativeSource(PathBuf::from("data/abstraction/orm.js"))
            );
        }

        #[test]
        fn test_resolves_from_asterisk_to_package_dependency() {
            let (project_dir, import_resolver) = create_import_resolver_for_fixture(
                "fixtures/node/js/subpath_import/from_asterisk/to_package_dependency",
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data.js",
                ),
                ModuleImport::PackageDependency {
                    package: "data-dep".to_string(),
                    subpath: None
                },
            );
        }

        #[test]
        fn test_resolves_from_asterisk_to_relative_source() {
            let (project_dir, import_resolver) = create_import_resolver_for_fixture(
                "fixtures/node/js/subpath_import/from_asterisk/to_relative_source",
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data.js",
                ),
                ModuleImport::RelativeSource(PathBuf::from("lib/redis.js"))
            );
        }
    }

    mod from_asterisk_extension {
        use super::*;

        #[test]
        #[ignore]
        fn test_resolves_from_asterisk_extension_to_asterisk() {
            let (project_dir, import_resolver) = create_import_resolver_for_fixture(
                "fixtures/node/js/subpath_import/from_asterisk_extension/_to_asterisk",
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data/raw.js",
                ),
                ModuleImport::RelativeSource(PathBuf::from("data/raw.js"))
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data/abstraction/orm.js",
                ),
                ModuleImport::RelativeSource(PathBuf::from("data/abstraction/orm.js"))
            );
        }

        #[test]
        fn test_resolves_from_asterisk_extension_to_asterisk_extension() {
            let (project_dir, import_resolver) = create_import_resolver_for_fixture(
                "fixtures/node/js/subpath_import/from_asterisk_extension/to_asterisk_extension",
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data/raw.js",
                ),
                ModuleImport::RelativeSource(PathBuf::from("data/raw.js"))
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data/abstraction/orm.js",
                ),
                ModuleImport::RelativeSource(PathBuf::from("data/abstraction/orm.js"))
            );
        }

        #[test]
        fn test_resolves_from_asterisk_extension_to_package_dependency() {
            let (project_dir, import_resolver) = create_import_resolver_for_fixture(
                "fixtures/node/js/subpath_import/from_asterisk_extension/to_package_dependency",
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data.js",
                ),
                ModuleImport::PackageDependency {
                    package: "data-dep".to_string(),
                    subpath: None
                }
            );
        }

        #[test]
        fn test_resolves_from_asterisk_extension_to_relative_source() {
            let (project_dir, import_resolver) = create_import_resolver_for_fixture(
                "fixtures/node/js/subpath_import/from_asterisk_extension/to_relative_source",
            );
            assert_eq!(
                import_resolver.resolve(
                    &project_dir,
                    &PathBuf::from("routes/data/lambda.js"),
                    "#lib/data.js",
                ),
                ModuleImport::RelativeSource(PathBuf::from("lib/redis.js"))
            );
        }
    }
}
