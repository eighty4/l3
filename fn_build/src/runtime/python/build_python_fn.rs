use crate::archive::write_archive;
use crate::runtime::python::parse_python_fn;
use crate::{
    BuildMode, FnBuildManifest, FnBuildOutput, FnBuildResult, FnBuildSpec, FnDependencies,
    FnHandler, FnSource,
};
use std::fs;
use std::path::Path;

pub async fn build_python_fn(build_spec: FnBuildSpec) -> FnBuildResult<FnBuildManifest> {
    let parse_manifest = parse_python_fn(build_spec.to_parse_spec()).await?;
    match parse_manifest.dependencies {
        FnDependencies::Unused => {}
        _ => todo!(),
    };
    let build_dir = build_spec.output.build_root.join(match build_spec.mode {
        BuildMode::Debug => "debug",
        BuildMode::Release => "release",
    });
    copy_sources(&build_spec.project_dir, &parse_manifest.sources, &build_dir)?;
    let archive_file = if build_spec.output.create_archive {
        let archive_path = build_spec.output.build_root.join(match build_spec.mode {
            BuildMode::Debug => "debug.zip",
            BuildMode::Release => "release.zip",
        });
        write_archive(
            &build_dir,
            &archive_path,
            &parse_manifest.sources,
            Vec::new(),
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

fn copy_sources(
    project_dir: &Path,
    sources: &Vec<FnSource>,
    build_dir: &Path,
) -> FnBuildResult<()> {
    for source in sources {
        let build_path = build_dir.join(&source.path);
        fs::create_dir_all(build_path.parent().unwrap())?;
        fs::copy(project_dir.join(&source.path), build_path)?;
    }
    Ok(())
}
