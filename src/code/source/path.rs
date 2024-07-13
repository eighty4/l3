use std::path::{Path, PathBuf};

use crate::code::source::{FunctionBuildDir, Language};

#[derive(Clone, PartialEq)]
pub enum SourceKind {
    FunctionBuild(FunctionBuildDir),
    OriginalSource,
}

#[derive(Clone)]
pub struct SourcePath {
    /// Distinguish between source roots for project sources and build outputs
    pub kind: SourceKind,
    /// Absolute path to source in filesystem
    pub abs: PathBuf,
    /// Relative path to source from project or build output root
    pub rel: PathBuf,
}

impl SourcePath {
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

    pub fn to_build_dir(&self, build_dir: FunctionBuildDir, project_dir: &Path) -> Self {
        let rel = match self.rel.extension().unwrap().to_string_lossy().as_ref() {
            "ts" => self.rel.with_extension("js"),
            _ => self.rel.clone(),
        };
        let abs = build_dir.abs(project_dir).join(&rel);
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
