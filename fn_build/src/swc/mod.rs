use crate::result::{FnBuild, FnBuildResult, FnSource, FnSources, ModuleImport};
use crate::spec::{BuildMode, FnBuildSpec, FnParseSpec};
use crate::swc::compiler::SwcCompiler;
use crate::swc::imports::{ImportResolver, NodeImportResolver};
use crate::swc::visitors::ImportVisitor;
use std::fs;
use std::path::Path;
use swc_ecma_visit::FoldWith;

mod compiler;
mod imports;
mod visitors;

#[cfg(test)]
mod imports_test;

#[cfg(test)]
mod swc_test;

#[cfg(test)]
mod visitors_test;

pub async fn build_js_fn(build_spec: FnBuildSpec) -> FnBuildResult<FnBuild> {
    let sources = parse_js_fn(FnParseSpec {
        entrypoint: build_spec.entrypoint.clone(),
        project_dir: build_spec.project_dir.clone(),
    })
    .await?;
    for source in &sources {
        debug_assert!(source.path.is_relative());
        let output_path = build_spec.output.join(&source.path);
        fs::create_dir_all(output_path.parent().unwrap()).expect("mkdir -p");
        match build_spec.mode {
            BuildMode::Debug => {
                fs::copy(build_spec.project_dir.join(&source.path), output_path).expect("cp");
            }
            BuildMode::Release => {
                let js_path = build_spec.project_dir.join(&source.path);
                let minified_js = SwcCompiler::new().minify_js(&js_path).unwrap();
                fs::write(output_path, minified_js).unwrap();
            }
        }
    }
    Ok(FnBuild {
        sources,
        output: build_spec.output,
    })
}

pub async fn parse_js_fn(parse_spec: FnParseSpec) -> FnBuildResult<FnSources> {
    let mut sources = Vec::new();
    let compiler = SwcCompiler::new();
    let entrypoint = parse_js(
        compiler.clone(),
        &parse_spec.project_dir,
        &parse_spec.entrypoint,
    )?;
    for import in &entrypoint.imports {
        match import {
            ModuleImport::RelativeSource(path) => {
                sources.push(parse_js(compiler.clone(), &parse_spec.project_dir, path)?);
            }
            _ => panic!(),
        }
    }
    sources.push(entrypoint);
    Ok(sources)
}

fn parse_js(compiler: SwcCompiler, project_dir: &Path, source: &Path) -> FnBuildResult<FnSource> {
    debug_assert!(project_dir.is_absolute());
    debug_assert!(project_dir.is_dir());
    debug_assert!(source.is_relative());
    let module = compiler.parse_es_module(&project_dir.join(source)).unwrap();
    let mut visitor = ImportVisitor::new();
    module.fold_with(&mut visitor);
    let import_resolver = NodeImportResolver {};
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
