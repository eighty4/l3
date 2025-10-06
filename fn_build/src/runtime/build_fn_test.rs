use std::{
    fs::{self, read_dir},
    path::{Path, PathBuf},
    sync::Arc,
};

use temp_dir::TempDir;

use crate::{
    runtime::{
        build_fn::{build_fn_inner, BuildTask, TransformResult},
        Runtime,
    },
    BuildMode, FnBuildResult, FnBuildSpec, FnDependencies, FnEntrypoint, FnOutputConfig,
    FnParseManifest, FnSource,
};

fn retaining_transform(_p: &Path, _s: String) -> FnBuildResult<TransformResult> {
    Ok(TransformResult::RetainPath("compiled".into()))
}

fn rewriting_transform(_p: &Path, _s: String) -> FnBuildResult<TransformResult> {
    Ok(TransformResult::RewriteExt("compiled".into(), "js".into()))
}

#[tokio::test]
async fn test_build_fn_represents_transform_with_retained_path() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.child("source.js"), "human friendly").unwrap();
    let build_spec = FnBuildSpec {
        entrypoint: PathBuf::new(),
        handler_fn_name: "PATCH".into(),
        mode: BuildMode::Debug,
        output: FnOutputConfig {
            build_root: temp_dir.child("build"),
            create_archive: false,
            dirname: "my-sweet-lambda".into(),
            use_build_mode: false,
        },
        project_dir: Arc::new(temp_dir.path().to_path_buf()),
        runtime: Runtime::Node(None),
    };
    let parse_manifest = FnParseManifest {
        dependencies: FnDependencies::Unused,
        entrypoint: FnEntrypoint {
            path: PathBuf::new(),
            handlers: Vec::new(),
        },
        sources: [FnSource {
            path: PathBuf::from("source.js"),
            imports: Vec::new(),
        }]
        .into(),
    };
    let build_tasks: Vec<BuildTask> = [BuildTask::TransformSourceFile("source.js".into())].into();
    let build_manifest = build_fn_inner(
        &build_spec,
        parse_manifest,
        build_tasks,
        retaining_transform,
    )
    .await
    .unwrap();
    let mut files = read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            entry
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect::<Vec<String>>();
    files.sort();
    assert_eq!(
        files,
        Vec::from([String::from("build"), String::from("source.js")])
    );
    files = read_dir(temp_dir.child("build").join("my-sweet-lambda"))
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            entry
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect::<Vec<String>>();
    assert_eq!(files, Vec::from([String::from("source.js")]));
    assert_eq!(
        fs::read_to_string(
            temp_dir
                .child("build")
                .join("my-sweet-lambda")
                .join("source.js")
        )
        .unwrap(),
        "compiled".to_string()
    );
    assert_eq!(build_manifest.sources.len(), 1);
    assert_eq!(
        build_manifest.sources.first().unwrap().path,
        PathBuf::from("source.js")
    );
    assert_eq!(build_manifest.output.paths.len(), 1);
    assert!(build_manifest
        .output
        .paths
        .contains_key(&PathBuf::from("source.js")));
    assert_eq!(
        build_manifest.output.paths.get(&PathBuf::from("source.js")),
        Some(&PathBuf::from("source.js"))
    );
}

#[tokio::test]
async fn test_build_fn_represents_transform_with_path_rewrite() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.child("source.ts"), "human friendly").unwrap();
    let build_spec = FnBuildSpec {
        entrypoint: PathBuf::new(),
        handler_fn_name: "PATCH".into(),
        mode: BuildMode::Debug,
        output: FnOutputConfig {
            build_root: temp_dir.child("build"),
            create_archive: false,
            dirname: "my-sweet-lambda".into(),
            use_build_mode: false,
        },
        project_dir: Arc::new(temp_dir.path().to_path_buf()),
        runtime: Runtime::Node(None),
    };
    let parse_manifest = FnParseManifest {
        dependencies: FnDependencies::Unused,
        entrypoint: FnEntrypoint {
            path: PathBuf::new(),
            handlers: Vec::new(),
        },
        sources: [FnSource {
            path: PathBuf::from("source.ts"),
            imports: Vec::new(),
        }]
        .into(),
    };
    let build_tasks: Vec<BuildTask> = [BuildTask::TransformSourceFile("source.ts".into())].into();
    let build_manifest = build_fn_inner(
        &build_spec,
        parse_manifest,
        build_tasks,
        rewriting_transform,
    )
    .await
    .unwrap();
    let mut files = read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            entry
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect::<Vec<String>>();
    files.sort();
    assert_eq!(
        files,
        Vec::from([String::from("build"), String::from("source.ts")])
    );
    files = read_dir(temp_dir.child("build").join("my-sweet-lambda"))
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            entry
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect::<Vec<String>>();
    files.sort();
    assert_eq!(files, Vec::from([String::from("source.js")]));
    assert_eq!(
        fs::read_to_string(
            temp_dir
                .child("build")
                .join("my-sweet-lambda")
                .join("source.js")
        )
        .unwrap(),
        "compiled".to_string()
    );
    assert_eq!(build_manifest.sources.len(), 1);
    assert_eq!(
        build_manifest.sources.first().unwrap().path,
        PathBuf::from("source.ts")
    );
    assert_eq!(build_manifest.output.paths.len(), 1);
    assert!(build_manifest
        .output
        .paths
        .contains_key(&PathBuf::from("source.ts")));
    assert_eq!(
        build_manifest.output.paths.get(&PathBuf::from("source.ts")),
        Some(&PathBuf::from("source.js"))
    );
}
