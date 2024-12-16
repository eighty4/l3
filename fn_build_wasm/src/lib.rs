wit_bindgen::generate!({
    world: "fn-parsing"
});

use l3_fn_build;
use std::path::PathBuf;
use std::sync::Arc;

struct FnParsingWasm;

impl Guest for FnParsingWasm {
    fn parse_entrypoint(spec: FnParseSpec) -> FnEntrypoint {
        Self::debug_print_spec(&spec);
        // l3_fn_build::parse_entrypoint(Self::map_fn_parse_spec(spec));
        FnEntrypoint {
            handlers: Vec::new(),
            path: "".to_string(),
        }
    }

    fn parse_fn(spec: FnParseSpec) -> FnParseManifest {
        Self::debug_print_spec(&spec);
        // l3_fn_build::parse_fn(Self::map_fn_parse_spec(spec));
        FnParseManifest {
            dependencies: FnDependencies::Required,
            entrypoint: FnEntrypoint {
                handlers: vec![],
                path: String::from("woo"),
            },
            sources: vec![],
        }
    }
}

impl FnParsingWasm {
    fn map_fn_parse_spec(spec: FnParseSpec) -> l3_fn_build::FnParseSpec {
        l3_fn_build::FnParseSpec {
            entrypoint: PathBuf::from(spec.entrypoint),
            project_dir: Arc::new(PathBuf::from(spec.project_dir)),
            runtime: match spec.runtime {
                Runtime::Node => l3_fn_build::runtime::Runtime::Node(Arc::new(
                    l3_fn_build::runtime::node::NodeConfig::default(),
                )),
                Runtime::Python => l3_fn_build::runtime::Runtime::Python,
            },
        }
    }

    fn debug_print_spec(spec: &FnParseSpec) {
        println!(
            "l3_fn_build::FnParseSpec project_dir={} entrypoint={} runtime={}",
            spec.project_dir,
            spec.entrypoint,
            match spec.runtime {
                Runtime::Node => "node",
                Runtime::Python => "python",
            }
        );
    }
}

export!(FnParsingWasm);
