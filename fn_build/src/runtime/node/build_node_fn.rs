use crate::archive::write_archive;
use crate::fs::copy_dir_all;
use crate::runtime::node::parse_node_fn;
use crate::swc::compiler::SwcCompiler;
use crate::{
    BuildMode, FnBuild, FnBuildOutput, FnBuildResult, FnBuildSpec, FnDependencies, FnSource,
};
use std::fs;
use std::path::Path;

pub async fn build_node_fn(build_spec: FnBuildSpec) -> FnBuildResult<FnBuild> {
    let manifest = parse_node_fn(build_spec.function.clone()).await?;
    let build_dir = match &build_spec.output {
        FnBuildOutput::Archive { build_root, .. } => build_root,
        FnBuildOutput::Directory(build_root) => build_root,
    }
    .join(match build_spec.mode {
        BuildMode::Debug => "debug",
        BuildMode::Release => "release",
    });
    if let FnDependencies::Required = manifest.dependencies {
        copy_dir_all(
            &build_spec.function.project_dir.join("node_modules"),
            &build_dir.join("node_modules"),
        )?;
    }
    for source in &manifest.sources {
        debug_assert!(source.path.is_relative());
        let output_path = build_dir.join(&source.path);
        fs::create_dir_all(output_path.parent().unwrap()).expect("mkdir -p");
        if let Some(extension) = source.path.extension() {
            if (extension == "js" || extension == "mjs") && build_spec.mode == BuildMode::Release {
                let js_path = build_spec.function.project_dir.join(&source.path);
                let minified_js = SwcCompiler::new().minify_js(&js_path).unwrap();
                fs::write(output_path, minified_js)?;
                continue;
            }
        }
        fs::copy(
            build_spec.function.project_dir.join(&source.path),
            output_path,
        )
        .expect("cp");
    }
    if let FnBuildOutput::Archive {
        build_root,
        archive_file,
    } = &build_spec.output
    {
        write_archive(&build_dir, &manifest, &build_root.join(archive_file)).unwrap();
    }
    Ok(FnBuild {
        manifest,
        output: build_spec.output,
    })
}

#[allow(unused)]
async fn copy_sources(
    project_dir: &Path,
    sources: &Vec<FnSource>,
    output_path: &Path,
) -> FnBuildResult<()> {
    for source in sources {
        fs::copy(project_dir.join(&source.path), output_path)?;
    }
    Ok(())
}
