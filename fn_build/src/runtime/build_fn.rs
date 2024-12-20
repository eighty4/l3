use crate::archive::write_archive;
use crate::checksum::Checksum;
use crate::{
    FnBuildError, FnBuildManifest, FnBuildOutput, FnBuildResult, FnBuildSpec, FnHandler,
    FnParseManifest,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task::JoinSet;

// todo include completed BuildTask for error reporting and tracing
struct BuildTaskResult {
    checksum: Option<Checksum>,
    path: PathBuf,
}

/// Compose work of a function build into model for parallel processing.
#[derive(Debug)]
pub enum BuildTask {
    /// Copies a directory recursively without creating checksums.
    CopyDirectoryRecursively(PathBuf),
    /// Copy a source file and get a checksum.
    CopySourceFile(PathBuf),
    /// Copy source files and get checksums.
    CopySourceFiles(Vec<PathBuf>),
    /// Perform a transform function on a source file and get a checksum of source input.
    TransformSourceFile(PathBuf),
}

pub async fn build_fn_inner<F>(
    build_spec: &FnBuildSpec,
    parse_manifest: FnParseManifest,
    build_tasks: Vec<BuildTask>,
    transform: F,
) -> FnBuildResult<FnBuildManifest>
where
    F: (Fn(&Path) -> FnBuildResult<String>) + Send + Sync + 'static,
{
    let fn_identifier = parse_manifest
        .entrypoint
        .to_fn_identifier(&build_spec.handler_fn_name)?;
    let build_root = build_spec.output_build_root();
    let build_dir = Arc::new(build_root.join(&fn_identifier));
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
            BuildTask::CopySourceFile(path) => {
                assert!(path.is_relative());
                _ = join_set.spawn(copy_source(
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
    while let Some(join_result) = join_set.join_next().await {
        let build_result = join_result.unwrap();
        match build_result {
            Ok(task_result) => {
                if let Some(checksum) = task_result.checksum {
                    checksums.insert(task_result.path, checksum);
                }
            }
            Err(build_err) => return Err(build_err),
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
                let archive_file = build_root.join(format!("{}.zip", fn_identifier));
                write_archive(&archive_file, &build_dir)?;
                Some(archive_file)
            } else {
                None
            },
            build_dir: build_dir.to_path_buf(),
        },
    })
}

async fn build_source<F>(
    project_dir: Arc<PathBuf>,
    build_dir: Arc<PathBuf>,
    path: PathBuf,
    transform: Arc<F>,
) -> FnBuildResult<BuildTaskResult>
where
    F: Fn(&Path) -> FnBuildResult<String>,
{
    let content = transform(&project_dir.join(&path))?;
    let checksum = Some(Checksum::try_from(content.as_str())?);
    let dest = build_dir.join(&path);
    _ = fs::create_dir_all(dest.parent().unwrap());
    match fs::write(build_dir.join(&path), content) {
        Ok(_) => Ok(BuildTaskResult { checksum, path }),
        Err(err) => Err(FnBuildError::IoError(err)),
    }
}

// todo parallelize
async fn copy_directory(
    project_dir: Arc<PathBuf>,
    build_dir: Arc<PathBuf>,
    path: PathBuf,
) -> FnBuildResult<BuildTaskResult> {
    for abs in l3_api_base::collect_files(&project_dir.join(&path)) {
        let rel = abs
            .strip_prefix(project_dir.as_path())
            .unwrap()
            .to_path_buf();
        let build_path = build_dir.join(&rel);
        // todo optimize collect_files to visitor pattern to only call create_dir_all once per dir
        fs::create_dir_all(build_path.parent().unwrap())?;
        fs::copy(project_dir.join(&rel), build_path)?;
    }
    Ok(BuildTaskResult {
        checksum: None,
        path,
    })
}

// todo refactor for copy_directory, currently copy_dir_all only works for rel paths
// pub fn copy_dir_all(from_dir: &Path, to: &Path) -> io::Result<()> {
//     debug_assert!(from_dir.is_dir());
//     create_dir_all(to)?;
//     for dir_entry_result in read_dir(from_dir)? {
//         let dir_entry = dir_entry_result?;
//         let from = dir_entry.path();
//         if from.is_dir() {
//             copy_dir_all(&from, &to.join(from.file_name().unwrap()))?;
//         } else {
//             copy(&from, to.join(from.file_name().unwrap()))?;
//         }
//     }
//     Ok(())
// }

async fn copy_source(
    project_dir: Arc<PathBuf>,
    build_dir: Arc<PathBuf>,
    path: PathBuf,
) -> FnBuildResult<BuildTaskResult> {
    let dest = build_dir.join(&path);
    _ = fs::create_dir_all(dest.parent().unwrap());
    let src = project_dir.join(&path);
    fs::copy(&src, &dest)?;
    let checksum = Some(Checksum::try_from(src.as_path())?);
    Ok(BuildTaskResult { checksum, path })
}
