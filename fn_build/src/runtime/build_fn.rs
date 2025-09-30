use crate::archive::write_archive;
use crate::checksum::Checksum;
use crate::{
    FnBuildManifest, FnBuildOutput, FnBuildResult, FnBuildSpec, FnHandler, FnParseManifest,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task::JoinSet;

// todo include completed BuildTask for error reporting and tracing
enum BuildTaskResult {
    Copied {
        checksum: Checksum,
        source_path: PathBuf,
    },
    Transformed {
        checksum: Checksum,
        source_path: PathBuf,
        /// None if source_path is used for path in build directory.
        output_path: Option<PathBuf>,
    },
    /// Specifies a BuildTask completed that isn't tracked to merge into FnBuildManifest
    Untracked,
}

/// Compose work of a function build into model for parallel processing.
#[derive(Debug)]
pub enum BuildTask {
    /// Copies a directory recursively without creating checksums.
    /// This task does not track checksums becaues it is currently
    /// used for copying node_modules.
    CopyDirectoryRecursively(PathBuf),
    /// Copy source files and get checksums.
    CopySourceFiles(Vec<PathBuf>),
    /// Perform a transform function on a source file and get a checksum of source input.
    TransformSourceFile(PathBuf),
}

/// Transform functions declaratively resolve output path.
pub enum TransformResult {
    /// Result is written to build dir retaining source path.
    RetainPath(String),
    /// Result is written to build dir with a new extension.
    RewriteExt(String, String),
}

pub async fn build_fn_inner<F>(
    build_spec: &FnBuildSpec,
    parse_manifest: FnParseManifest,
    build_tasks: Vec<BuildTask>,
    transform: F,
) -> FnBuildResult<FnBuildManifest>
where
    F: (Fn(&Path, String) -> FnBuildResult<TransformResult>) + Send + Sync + 'static,
{
    let build_root = build_spec.output_build_root();
    let build_dir = Arc::new(build_root.join(&build_spec.output.dirname));
    let transform = Arc::new(transform);
    let mut join_set: JoinSet<FnBuildResult<BuildTaskResult>> = JoinSet::new();
    for build_task in build_tasks {
        match build_task {
            BuildTask::CopyDirectoryRecursively(path) => {
                assert!(path.is_relative());
                _ = join_set.spawn(copy_directory(
                    build_spec.project_dir.clone(),
                    build_dir.clone(),
                    path,
                ))
            }
            BuildTask::CopySourceFiles(paths) => {
                for path in paths {
                    assert!(path.is_relative());
                    _ = join_set.spawn(copy_source(
                        build_spec.project_dir.clone(),
                        build_dir.clone(),
                        path,
                    ))
                }
            }
            BuildTask::TransformSourceFile(path) => {
                assert!(path.is_relative());
                _ = join_set.spawn(build_source(
                    build_spec.project_dir.clone(),
                    build_dir.clone(),
                    path,
                    transform.clone(),
                ))
            }
        }
    }
    let mut checksums = HashMap::new();
    let mut paths = HashMap::new();
    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Err(join_err) => panic!("panic in build task: {}", join_err),
            Ok(build_task_result) => match build_task_result {
                Ok(BuildTaskResult::Copied {
                    checksum,
                    source_path,
                }) => {
                    paths.insert(source_path.clone(), source_path.clone());
                    checksums.insert(source_path, checksum);
                }
                Ok(BuildTaskResult::Transformed {
                    checksum,
                    source_path,
                    output_path,
                }) => {
                    match output_path {
                        Some(output_path) => {
                            debug_assert!(output_path.is_relative());
                            paths.insert(source_path.clone(), output_path)
                        }
                        None => paths.insert(source_path.clone(), source_path.clone()),
                    };
                    checksums.insert(source_path, checksum);
                }
                Ok(BuildTaskResult::Untracked) => {}
                Err(build_err) => return Err(build_err),
            },
        }
    }
    let handler = FnHandler::from_handler_fn(
        &parse_manifest.entrypoint.path,
        build_spec.handler_fn_name.clone(),
    );
    Ok(FnBuildManifest {
        checksums,
        dependencies: parse_manifest.dependencies,
        entrypoint: parse_manifest.entrypoint.path,
        sources: parse_manifest.sources,
        handler,
        output: FnBuildOutput {
            archive_file: if build_spec.output.create_archive {
                let archive_file = build_root.join(format!("{}.zip", &build_spec.output.dirname));
                write_archive(&archive_file, &build_dir)?;
                Some(archive_file)
            } else {
                None
            },
            build_dir: build_dir.to_path_buf(),
            paths,
        },
    })
}

async fn build_source<F>(
    project_dir: Arc<PathBuf>,
    build_dir: Arc<PathBuf>,
    source_path: PathBuf,
    transform: Arc<F>,
) -> FnBuildResult<BuildTaskResult>
where
    F: (Fn(&Path, String) -> FnBuildResult<TransformResult>) + Send + Sync + 'static,
{
    let abs_source_path = project_dir.join(&source_path);
    let source_content = fs::read_to_string(&abs_source_path)?;
    let checksum = Checksum::try_from(source_content.as_str())?;
    let (output_path, content) = match transform(&abs_source_path, source_content)? {
        TransformResult::RetainPath(content) => (None, content),
        TransformResult::RewriteExt(content, ext) => {
            (Some(source_path.with_extension(ext)), content)
        }
    };
    let dest = match &output_path {
        Some(output_path) => build_dir.join(output_path),
        None => build_dir.join(&source_path),
    };
    _ = fs::create_dir_all(dest.parent().unwrap());
    fs::write(dest, content)?;
    Ok(BuildTaskResult::Transformed {
        checksum,
        source_path,
        output_path,
    })
}

async fn copy_source(
    project_dir: Arc<PathBuf>,
    build_dir: Arc<PathBuf>,
    source_path: PathBuf,
) -> FnBuildResult<BuildTaskResult> {
    let dest = build_dir.join(&source_path);
    _ = fs::create_dir_all(dest.parent().unwrap());
    let source_content = fs::read_to_string(project_dir.join(&source_path))?;
    let checksum = Checksum::try_from(source_content.as_str())?;
    fs::write(&dest, &source_content)?;
    Ok(BuildTaskResult::Copied {
        checksum,
        source_path,
    })
}

// todo parallelize
async fn copy_directory(
    project_dir: Arc<PathBuf>,
    build_dir: Arc<PathBuf>,
    source_path: PathBuf,
) -> FnBuildResult<BuildTaskResult> {
    for abs in l3_api_base::collect_files(&project_dir.join(&source_path)) {
        let rel = abs
            .strip_prefix(project_dir.as_path())
            .unwrap()
            .to_path_buf();
        let build_path = build_dir.join(&rel);
        // todo optimize collect_files to visitor pattern to only call create_dir_all once per dir
        fs::create_dir_all(build_path.parent().unwrap())?;
        fs::copy(project_dir.join(&rel), build_path)?;
    }
    Ok(BuildTaskResult::Untracked)
}
