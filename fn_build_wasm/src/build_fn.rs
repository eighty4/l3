#![expect(clippy::too_many_arguments, reason = "generated code")]
wit_bindgen::generate!({
    world: "fn-building"
});

use std::path::PathBuf;
use std::sync::Arc;

use l3_fn_build::FnOutputConfig;

use crate::build_fn::l3::fn_build::build_result::{FnBuildOutput, SourceChecksum};
use crate::build_fn::l3::fn_build::build_spec::{BuildMode, Runtime};
use crate::build_fn::l3::fn_build::parse_result::{
    DependencyImport, FnDependencies, FnSource, ModuleImport,
};

struct FnBuildingWasm;

impl Guest for FnBuildingWasm {
    fn build_fn(spec: FnBuildSpec) -> Result<FnBuildManifest, String> {
        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("current thread runtime for l3_fn_build::parse_entrypoint")
            .block_on(l3_fn_build::build_fn(map_in_fn_build_spec(spec)));
        match result {
            Ok(val) => Ok(map_out_fn_build_manifest(val)),
            Err(err) => Err(err.to_string()),
        }
    }
}

fn map_in_fn_build_spec(build_spec: FnBuildSpec) -> l3_fn_build::FnBuildSpec {
    l3_fn_build::FnBuildSpec {
        entrypoint: PathBuf::from(build_spec.entrypoint),
        handler_fn_name: build_spec.handler_fn_name,
        mode: match build_spec.mode {
            BuildMode::Debug => l3_fn_build::BuildMode::Debug,
            BuildMode::Release => l3_fn_build::BuildMode::Release,
        },
        project_dir: Arc::new(PathBuf::from(build_spec.project_dir)),
        runtime: match build_spec.runtime {
            Runtime::Node => l3_fn_build::runtime::Runtime::Node(None),
            Runtime::Python => l3_fn_build::runtime::Runtime::Python,
        },
        output: FnOutputConfig {
            build_root: PathBuf::from(build_spec.output.build_root),
            create_archive: build_spec.output.create_archive,
            dirname: build_spec.output.dirname,
            use_build_mode: build_spec.output.use_build_mode,
        },
    }
}

fn map_out_fn_build_manifest(build_manifest: l3_fn_build::FnBuildManifest) -> FnBuildManifest {
    FnBuildManifest {
        entrypoint: build_manifest.entrypoint.to_string_lossy().to_string(),
        handler: build_manifest.handler.fn_name,
        dependencies: match build_manifest.dependencies {
            l3_fn_build::FnDependencies::Required => FnDependencies::Required,
            l3_fn_build::FnDependencies::Unused => FnDependencies::Unused,
        },
        checksums: build_manifest
            .checksums
            .into_iter()
            .map(|(path, checksum)| SourceChecksum {
                checksum: checksum.as_str().to_string(),
                path: path.to_string_lossy().to_string(),
            })
            .collect(),
        output: FnBuildOutput {
            build_dir: build_manifest
                .output
                .build_dir
                .to_string_lossy()
                .to_string(),
            paths: build_manifest
                .output
                .paths
                .into_iter()
                .map(|(src_path, out_path)| {
                    (
                        src_path.to_string_lossy().to_string(),
                        out_path.to_string_lossy().to_string(),
                    )
                })
                .collect(),
            archive_file: build_manifest
                .output
                .archive_file
                .map(|p| p.to_string_lossy().to_string()),
        },
        sources: build_manifest
            .sources
            .into_iter()
            .map(|source| FnSource {
                path: source.path.to_string_lossy().to_string(),
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
            })
            .collect(),
    }
}

export!(FnBuildingWasm);
