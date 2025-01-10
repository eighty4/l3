wit_bindgen::generate!({
    world: "fn-parsing"
});

use std::path::PathBuf;
use std::sync::Arc;

struct FnParsingWasm;

impl Guest for FnParsingWasm {
    fn parse_entrypoint(spec: FnParseSpec) -> Result<FnEntrypoint, String> {
        Self::debug_print_spec(&spec);
        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("current thread runtime for l3_fn_build::parse_entrypoint")
            .block_on(l3_fn_build::parse_entrypoint(Self::map_in_fn_parse_spec(
                spec,
            )));
        match result {
            Ok(val) => Ok(Self::map_out_fn_entrypoint(val)),
            Err(err) => Err(err.to_string()),
        }
    }

    fn parse_fn(spec: FnParseSpec) -> Result<FnParseManifest, String> {
        Self::debug_print_spec(&spec);
        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("current thread runtime for l3_fn_build::parse_fn")
            .block_on(l3_fn_build::parse_fn(Self::map_in_fn_parse_spec(spec)));
        match result {
            Ok(val) => Ok(Self::map_out_fn_parse_manifest(val)),
            Err(err) => Err(err.to_string()),
        }
    }
}

impl FnParsingWasm {
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
            entrypoint: Self::map_out_fn_entrypoint(parse_manifest.entrypoint),
            sources: parse_manifest
                .sources
                .into_iter()
                .map(Self::map_out_fn_source)
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
                    l3_fn_build::ModuleImport::Unknown(specifier) => {
                        ModuleImport::Unknown(specifier)
                    }
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
                .map(Self::map_out_fn_handler)
                .collect(),
            path: entrypoint.path.to_string_lossy().to_string(),
        }
    }

    fn map_out_fn_handler(handler: l3_fn_build::FnHandler) -> FnHandler {
        FnHandler {
            fn_name: handler.fn_name,
            routing: match handler.routing {
                l3_fn_build::FnRouting::HttpRoute(http_route) => FnRouting::HttpRoute(HttpRoute {
                    method: match http_route.method {
                        l3_fn_build::HttpMethod::Get => HttpMethod::Get,
                        l3_fn_build::HttpMethod::Delete => HttpMethod::Delete,
                        l3_fn_build::HttpMethod::Patch => HttpMethod::Patch,
                        l3_fn_build::HttpMethod::Post => HttpMethod::Post,
                        l3_fn_build::HttpMethod::Put => HttpMethod::Put,
                    },
                    path: http_route.path,
                }),
                l3_fn_build::FnRouting::Unsupported => FnRouting::Unsupported,
            },
        }
    }
}

export!(FnParsingWasm);
