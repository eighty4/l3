wit_bindgen::generate!({
    world: "fn-parsing"
});

use crate::parse_fn::l3::fn_build::parse_result::*;
use crate::parse_fn::l3::fn_build::parse_spec::*;

use std::path::PathBuf;
use std::sync::Arc;

struct FnParsingWasm;

impl Guest for FnParsingWasm {
    fn parse_entrypoint(spec: FnParseSpec) -> Result<FnEntrypoint, String> {
        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("current thread runtime for l3_fn_build::parse_entrypoint")
            .block_on(l3_fn_build::parse_entrypoint(map_in_fn_parse_spec(spec)));
        match result {
            Ok(val) => Ok(map_out_fn_entrypoint(val)),
            Err(err) => Err(err.to_string()),
        }
    }

    fn parse_fn(spec: FnParseSpec) -> Result<FnParseManifest, String> {
        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("current thread runtime for l3_fn_build::parse_fn")
            .block_on(l3_fn_build::parse_fn(map_in_fn_parse_spec(spec)));
        match result {
            Ok(val) => Ok(map_out_fn_parse_manifest(val)),
            Err(err) => Err(err.to_string()),
        }
    }
}

fn map_in_fn_parse_spec(parse_spec: FnParseSpec) -> l3_fn_build::FnParseSpec {
    l3_fn_build::FnParseSpec {
        entrypoint: PathBuf::from(parse_spec.entrypoint),
        project_dir: Arc::new(PathBuf::from(parse_spec.project_dir)),
        runtime: match parse_spec.runtime {
            Runtime::Node => l3_fn_build::runtime::Runtime::Node(None),
            Runtime::Python => l3_fn_build::runtime::Runtime::Python,
        },
    }
}

fn map_out_fn_parse_manifest(parse_manifest: l3_fn_build::FnParseManifest) -> FnParseManifest {
    FnParseManifest {
        dependencies: match parse_manifest.dependencies {
            l3_fn_build::FnDependencies::Required => FnDependencies::Required,
            l3_fn_build::FnDependencies::Unused => FnDependencies::Unused,
        },
        entrypoint: map_out_fn_entrypoint(parse_manifest.entrypoint),
        sources: parse_manifest
            .sources
            .into_iter()
            .map(map_out_fn_source)
            .collect(),
    }
}

fn map_out_fn_source(source: l3_fn_build::FnSource) -> FnSource {
    FnSource {
        imports: source
            .imports
            .into_iter()
            .map(|import| match import {
                l3_fn_build::ModuleImport::PackageDependency { package, subpath } => {
                    ModuleImport::PackageDependency(DependencyImport { package, subpath })
                }
                l3_fn_build::ModuleImport::RelativeSource(path) => {
                    ModuleImport::RelativeSource(path.to_string_lossy().to_string())
                }
                l3_fn_build::ModuleImport::Unknown(specifier) => ModuleImport::Unknown(specifier),
            })
            .collect(),
        path: source.path.to_string_lossy().to_string(),
    }
}

fn map_out_fn_entrypoint(entrypoint: l3_fn_build::FnEntrypoint) -> FnEntrypoint {
    FnEntrypoint {
        handlers: entrypoint
            .handlers
            .into_iter()
            .map(|fn_handler| fn_handler.fn_name)
            .collect(),
        source: entrypoint.path.to_string_lossy().to_string(),
    }
}

export!(FnParsingWasm);
