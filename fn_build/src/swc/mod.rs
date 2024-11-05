use crate::result::{FnBuild, FnBuildError, FnSource, ModuleImport};
use crate::spec::{BuildMode, FnBuildSpec, FnParseSpec};
use crate::swc::compiler::SwcCompiler;
use crate::swc::visitors::ImportVisitor;
use std::fs;
use swc_ecma_visit::FoldWith;

mod compiler;
mod visitors;

#[cfg(test)]
mod swc_test;

#[cfg(test)]
mod visitors_test;

pub async fn build_js_fn(build_spec: FnBuildSpec) -> Result<FnBuild, FnBuildError> {
    let output_file = build_spec.output.join(&build_spec.entrypoint);
    fs::create_dir_all(output_file.parent().unwrap()).expect("mkdir -p");
    match build_spec.mode {
        BuildMode::Debug => {
            fs::copy(
                build_spec.project_dir.join(&build_spec.entrypoint),
                output_file,
            )
            .expect("cp");
        }
        BuildMode::Release => {
            let js_path = build_spec.project_dir.join(&build_spec.entrypoint);
            let minified_js = SwcCompiler::new().minify_js(&js_path).unwrap();
            fs::write(output_file, minified_js).unwrap();
        }
    }
    Ok(FnBuild {
        entrypoint: FnSource {
            imports: Vec::new(),
            path: build_spec.entrypoint,
        },
        output: build_spec.output,
    })
}

pub async fn parse_js_fn(parse_spec: FnParseSpec) -> Result<FnSource, FnBuildError> {
    let compiler = SwcCompiler::new();
    let module = compiler
        .parse_es_module(&parse_spec.project_dir.join(&parse_spec.entrypoint))
        .unwrap();
    let mut visitor = ImportVisitor::new();
    module.fold_with(&mut visitor);
    let imports = visitor
        .result()
        .into_iter()
        .map(ModuleImport::Unknown)
        .collect();
    Ok(FnSource {
        imports,
        path: parse_spec.entrypoint,
    })
}
