use crate::fs::copy_dir_all;
use crate::result::{FnBuild, FnBuildResult, FnDependencies, FnManifest, FnSource, ModuleImport};
use crate::runtime::node::NodeImportResolver;
use crate::runtime::{ImportResolver, Runtime};
use crate::spec::{BuildMode, FnBuildSpec, FnParseSpec};
use crate::swc::compiler::SwcCompiler;
use crate::swc::visitors::ImportVisitor;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use swc_ecma_visit::FoldWith;

mod compiler;
mod visitors;

#[cfg(test)]
mod visitors_test;

pub async fn build_js_fn(build_spec: FnBuildSpec) -> FnBuildResult<FnBuild> {
    let manifest = parse_js_fn_inner(&build_spec.function).await?;
    if let FnDependencies::Required = manifest.dependencies {
        copy_dir_all(
            &build_spec.function.project_dir.join("node_modules"),
            &build_spec.output.join("node_modules"),
        )?;
    }
    for source in &manifest.sources {
        debug_assert!(source.path.is_relative());
        let output_path = build_spec.output.join(&source.path);
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
    Ok(FnBuild {
        manifest,
        output: build_spec.output,
    })
}

pub async fn parse_js_fn(parse_spec: FnParseSpec) -> FnBuildResult<FnManifest> {
    parse_js_fn_inner(&parse_spec).await
}

async fn parse_js_fn_inner(parse_spec: &FnParseSpec) -> FnBuildResult<FnManifest> {
    let Runtime::Node(node_config) = &parse_spec.runtime;
    let import_resolver = Arc::new(NodeImportResolver::new(node_config.clone()));
    let mut sources = Vec::new();
    let compiler = SwcCompiler::new();
    let entrypoint = parse_js(
        import_resolver.clone(),
        compiler.clone(),
        &parse_spec.project_dir,
        &parse_spec.entrypoint,
    )?;
    let mut requires_deps = false;
    for import in &entrypoint.imports {
        match import {
            ModuleImport::PackageDependency { .. } => requires_deps = true,
            ModuleImport::RelativeSource(path) => {
                sources.push(parse_js(
                    import_resolver.clone(),
                    compiler.clone(),
                    &parse_spec.project_dir,
                    path,
                )?);
            }
            ModuleImport::Unknown(_) => panic!(),
        }
    }
    sources.push(entrypoint);
    let package_json_path = PathBuf::from("package.json");
    if parse_spec.project_dir.join(&package_json_path).is_file() {
        sources.push(FnSource::from(package_json_path));
    }
    Ok(FnManifest {
        dependencies: if requires_deps {
            FnDependencies::Required
        } else {
            FnDependencies::Unused
        },
        sources,
    })
}

fn parse_js(
    import_resolver: Arc<NodeImportResolver>,
    compiler: SwcCompiler,
    project_dir: &Path,
    source: &Path,
) -> FnBuildResult<FnSource> {
    debug_assert!(project_dir.is_absolute());
    debug_assert!(project_dir.is_dir());
    debug_assert!(source.is_relative());
    let module = compiler.parse_es_module(&project_dir.join(source)).unwrap();
    let mut visitor = ImportVisitor::new();
    module.fold_with(&mut visitor);
    let imports = visitor
        .result()
        .into_iter()
        .map(|import| import_resolver.resolve(project_dir, source, import.as_str()))
        .collect();
    Ok(FnSource {
        imports,
        path: source.to_path_buf(),
    })
}
