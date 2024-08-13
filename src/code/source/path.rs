use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::code::build::BuildMode;
use crate::code::source::Language;
use crate::project::Lx3ProjectDeets;

/// FunctionBuildDir paths are tokenized by the build's cloud destination (AWS), API Gateway ID,
/// Lambda function name and BuildMode.
#[derive(Clone)]
pub struct FunctionBuildDir {
    pub abs: PathBuf,
    /// Relative to project_dir
    pub _rel: PathBuf,
}

impl FunctionBuildDir {
    pub fn new(project_deets: &Arc<Lx3ProjectDeets>, fn_name: &String) -> Self {
        let _rel = PathBuf::from(".l3")
            .join("aws")
            .join(&project_deets.aws.api.id)
            .join(fn_name)
            .join(match project_deets.build_mode {
                BuildMode::Debug => "debug",
                BuildMode::Release => "release",
            });
        let abs = project_deets.project_dir.join(&_rel);
        Self { abs, _rel }
    }
}

#[derive(Clone)]
pub enum SourceKind {
    /// FunctionBuild is a SourcePath to a function build output with an absolute path to build directory.
    FunctionBuild(FunctionBuildDir),
    /// OriginalSource is a SourcePath to a function source used as is without processing from the original path.
    OriginalSource,
}

#[derive(Clone)]
pub struct SourcePath {
    /// SourceKind distinguishes between source roots for project sources and build outputs.
    pub kind: SourceKind,
    /// Absolute path to source in filesystem
    pub abs: PathBuf,
    /// Relative path to source from project or build output root
    pub rel: PathBuf,
}

impl SourcePath {
    pub fn is_lambda_file_name(p: &Path) -> bool {
        matches!(
            p.file_name().unwrap().to_string_lossy().as_ref(),
            "lambda.js" | "lambda.mjs" | "lambda.py" | "lambda.ts"
        )
    }

    fn new(kind: SourceKind, abs: PathBuf, rel: PathBuf) -> Self {
        debug_assert!(abs.is_absolute());
        debug_assert!(rel.is_relative());
        Self { kind, abs, rel }
    }

    pub fn from_abs(project_dir: &Path, abs: PathBuf) -> Self {
        debug_assert!(abs.is_absolute());
        let rel = abs.strip_prefix(project_dir).unwrap().to_path_buf();
        Self::new(SourceKind::OriginalSource, abs, rel)
    }

    pub fn from_rel(project_dir: &Path, rel: PathBuf) -> Self {
        debug_assert!(rel.is_relative());
        let abs = project_dir.join(&rel);
        Self::new(SourceKind::OriginalSource, abs, rel)
    }

    pub fn file_name(&self) -> String {
        self.rel.file_name().unwrap().to_string_lossy().to_string()
    }

    pub fn language(&self) -> Option<Language> {
        Language::from_extension(&self.rel)
    }

    pub fn to_build_dir(&self, build_dir: FunctionBuildDir) -> Self {
        let rel = match self.rel.extension().unwrap().to_string_lossy().as_ref() {
            "ts" => self.rel.with_extension("js"),
            _ => self.rel.clone(),
        };
        let abs = build_dir.abs.join(&rel);
        Self::new(SourceKind::FunctionBuild(build_dir), abs, rel)
    }

    pub fn to_relative_source(&self, path: &Path) -> Self {
        let project_dir = PathBuf::from(
            self.abs
                .to_string_lossy()
                .strip_suffix(&self.rel.to_string_lossy().to_string())
                .unwrap(),
        );
        SourcePath::from_abs(
            &project_dir,
            match self.abs.file_name() {
                None => &self.abs,
                Some(_) => self.abs.parent().unwrap(),
            }
            .join(path),
        )
    }

    // todo rewrite . and .. path parts
    // pub fn to_relative_source(&self, path: &PathBuf) -> Result<Self, anyhow::Error> {
    //     debug_assert!(path.starts_with(".") || path.starts_with(".."));
    //     let project_dir = PathBuf::from(
    //         self.abs
    //             .to_string_lossy()
    //             .strip_suffix(&self.rel.to_string_lossy().to_string())
    //             .unwrap(),
    //     );
    //     let mut prefix = match self.abs.file_name() {
    //         None => &self.abs,
    //         Some(_) => self.abs.parent().unwrap(),
    //     };
    //     let mut suffix = PathBuf::new();
    //     for component in path.components() {
    //         match component.as_os_str().to_string_lossy().as_ref() {
    //             "." => continue,
    //             ".." => prefix = prefix.parent().unwrap(),
    //             &_ => suffix.push(component),
    //         }
    //     }
    //     Ok(SourcePath::from_abs(&project_dir, prefix.join(suffix)))
    // }
}
