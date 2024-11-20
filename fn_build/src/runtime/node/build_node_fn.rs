use crate::archive::{write_archive, ArchiveInclusion};
use crate::fs::copy_dir_all;
use crate::runtime::node::parse_node_fn;
use crate::swc::compiler::SwcCompiler;
use crate::{
    BuildMode, FnBuildManifest, FnBuildOutput, FnBuildResult, FnBuildSpec, FnDependencies,
    FnHandler,
};
use std::fs;
use std::path::PathBuf;

pub async fn build_node_fn(build_spec: FnBuildSpec) -> FnBuildResult<FnBuildManifest> {
    let parse_manifest = parse_node_fn(build_spec.to_parse_spec()).await?;
    let build_dir = build_spec.output.build_root.join(match build_spec.mode {
        BuildMode::Debug => "debug",
        BuildMode::Release => "release",
    });
    if let FnDependencies::Required = parse_manifest.dependencies {
        copy_dir_all(
            &build_spec.project_dir.join("node_modules"),
            &build_dir.join("node_modules"),
        )?;
    }
    for source in &parse_manifest.sources {
        debug_assert!(source.path.is_relative());
        let output_path = build_dir.join(&source.path);
        fs::create_dir_all(output_path.parent().unwrap()).expect("mkdir -p");
        if let Some(extension) = source.path.extension() {
            if (extension == "js" || extension == "mjs") && build_spec.mode == BuildMode::Release {
                let js_path = build_spec.project_dir.join(&source.path);
                let minified_js = SwcCompiler::new().minify_js(&js_path).unwrap();
                fs::write(output_path, minified_js)?;
                continue;
            }
        }
        fs::copy(build_spec.project_dir.join(&source.path), output_path).expect("cp");
    }
    let archive_file = if build_spec.output.create_archive {
        let archive_path = build_spec.output.build_root.join(match build_spec.mode {
            BuildMode::Debug => "debug.zip",
            BuildMode::Release => "release.zip",
        });
        write_archive(
            &build_dir,
            &archive_path,
            &parse_manifest.sources,
            match parse_manifest.dependencies {
                FnDependencies::Required => {
                    vec![ArchiveInclusion::Directory(PathBuf::from("node_modules"))]
                }
                FnDependencies::Unused => Vec::new(),
            },
        )
        .unwrap();
        Some(archive_path)
    } else {
        None
    };
    let handler = FnHandler::from_handler_fn(
        &parse_manifest.entrypoint,
        build_spec.handler_fn_name.clone(),
    );
    Ok(FnBuildManifest {
        dependencies: parse_manifest.dependencies,
        entrypoint: parse_manifest.entrypoint,
        sources: parse_manifest.sources,
        handler,
        output: FnBuildOutput {
            archive_file,
            build_dir,
        },
    })
}
