use crate::runtime::node::imports::subpath::read::read_subpath_imports;
use crate::runtime::node::imports::subpath::NodeSubpathImports;
use crate::runtime::node::imports::{
    NodeSubpathImportAsterisks, NodeSubpathImportMapWildcardTo, NodeSubpathImportMapping,
    NodeSubpathImportWildcard,
};
use serde_json::{Map, Value};

fn parse_for_read_result(json: &str) -> NodeSubpathImports {
    read_subpath_imports(&serde_json::from_str::<Map<String, Value>>(json).unwrap()).unwrap()
}

#[test]
pub fn test_read_subpath_imports_parses_explicit_subpath_import() {
    let subpath_imports = parse_for_read_result(
        r##"{
            "#lib/data.js": "./lib/data.js"
        }"##,
    );
    assert_eq!(1, subpath_imports.len());
    assert_eq!(
        &NodeSubpathImportMapping::Explicit {
            from: "#lib/data.js".to_string(),
            to: "./lib/data.js".to_string()
        },
        subpath_imports.first().unwrap()
    );
}

#[test]
pub fn test_read_subpath_imports_parses_wildcard_subpath_import() {
    let subpath_imports = parse_for_read_result(
        r##"{
            "#lib/*": "./lib/*"
        }"##,
    );
    assert_eq!(1, subpath_imports.len());
    assert_eq!(
        &NodeSubpathImportMapping::Wildcard {
            from: NodeSubpathImportWildcard {
                before: "#lib/".to_string(),
                after: None,
            },
            to: NodeSubpathImportMapWildcardTo::Wildcard(
                "./lib/*".to_string(),
                NodeSubpathImportAsterisks::Single
            ),
        },
        subpath_imports.first().unwrap()
    );
}

#[test]
pub fn test_read_subpath_imports_parses_wildcard_subpath_import_with_extension() {
    let subpath_imports = parse_for_read_result(
        r##"{
            "#lib/*.js": "./lib/*.js"
        }"##,
    );
    assert_eq!(1, subpath_imports.len());
    assert_eq!(
        &NodeSubpathImportMapping::Wildcard {
            from: NodeSubpathImportWildcard {
                before: "#lib/".to_string(),
                after: Some(".js".to_string()),
            },
            to: NodeSubpathImportMapWildcardTo::Wildcard(
                "./lib/*.js".to_string(),
                NodeSubpathImportAsterisks::Single
            ),
        },
        subpath_imports.first().unwrap()
    );
}

#[test]
pub fn test_read_subpath_imports_parses_wildcard_subpath_import_with_multiple_asterisks() {
    let subpath_imports = parse_for_read_result(
        r##"{
            "#lib/*.js": "./lib/*/*.js"
        }"##,
    );
    assert_eq!(1, subpath_imports.len());
    assert_eq!(
        &NodeSubpathImportMapping::Wildcard {
            from: NodeSubpathImportWildcard {
                before: "#lib/".to_string(),
                after: Some(".js".to_string()),
            },
            to: NodeSubpathImportMapWildcardTo::Wildcard(
                "./lib/*/*.js".to_string(),
                NodeSubpathImportAsterisks::Multiple
            ),
        },
        subpath_imports.first().unwrap()
    );
}

#[test]
pub fn test_read_subpath_imports_parses_wildcard_subpath_import_mapped_to_explicit_path() {
    let subpath_imports = parse_for_read_result(
        r##"{
            "#lib/*.js": "./lib/data.js"
        }"##,
    );
    assert_eq!(1, subpath_imports.len());
    assert_eq!(
        &NodeSubpathImportMapping::Wildcard {
            from: NodeSubpathImportWildcard {
                before: "#lib/".to_string(),
                after: Some(".js".to_string()),
            },
            to: NodeSubpathImportMapWildcardTo::Explicit("./lib/data.js".to_string()),
        },
        subpath_imports.first().unwrap()
    );
}

#[test]
pub fn test_read_subpath_imports_removes_cjs_conditional_subpath_imports() {
    let subpath_imports = parse_for_read_result(
        r##"{
            "#lib/*": {
                "require": "./lib/cjs/*",
                "module-sync": "./lib/cjs/*",
                "import": "./lib/mjs/*"
            }
        }"##,
    );
    assert_eq!(1, subpath_imports.len());
    assert_eq!(
        &NodeSubpathImportMapping::Wildcard {
            from: NodeSubpathImportWildcard {
                before: "#lib/".to_string(),
                after: None,
            },
            to: NodeSubpathImportMapWildcardTo::Wildcard(
                "./lib/mjs/*".to_string(),
                NodeSubpathImportAsterisks::Single
            ),
        },
        subpath_imports.first().unwrap()
    );
}

#[test]
pub fn test_read_subpath_imports_resolves_to_first_es_conditional_subpath_imports() {
    let subpath_imports = parse_for_read_result(
        r##"{
            "#data": {
                "default": {
                    "import": {
                        "require": {
                            "default": "./lib/data-polyfill.js"
                        }
                    }
                },
                "node": "data-dep"
            }
        }"##,
    );
    assert_eq!(1, subpath_imports.len());
    assert_eq!(
        &NodeSubpathImportMapping::Explicit {
            from: "#data".to_string(),
            to: "data-dep".to_string()
        },
        subpath_imports.first().unwrap()
    );
}
