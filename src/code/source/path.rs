use crate::code::build::BuildMode;
use crate::code::source::Language;
use crate::lambda::LambdaFn;
use crate::project::Lx3Project;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR};
use std::sync::Arc;

#[derive(Clone)]
pub enum FunctionBuildDir {
    /// PlatformSynced build directories are tokenized by the build's cloud
    /// destination (AWS), API Gateway ID, Lambda function name and BuildMode.
    PlatformSync(Arc<Lx3Project>, Arc<LambdaFn>),
}

impl FunctionBuildDir {
    pub fn to_path(&self) -> PathBuf {
        match self {
            FunctionBuildDir::PlatformSync(project, lambda_fn) => project.dir.join(
                PathBuf::from(".l3")
                    .join("aws")
                    .join(&project.aws().api.id)
                    .join(&lambda_fn.fn_name)
                    .join(match project.build_mode {
                        BuildMode::Debug => "debug",
                        BuildMode::Release => "release",
                    }),
            ),
        }
    }
}

#[derive(Clone)]
pub enum SourceKind {
    /// FunctionBuild is a SourcePath to a function build output with an absolute path to build directory.
    #[allow(unused)]
    FunctionBuild(FunctionBuildDir),
    /// OriginalSource is a SourcePath to a function source used as is without processing from the original path.
    OriginalSource,
}

#[derive(Clone)]
pub struct SourcePath {
    /// SourceKind distinguishes between source roots for project sources and build outputs.
    pub kind: SourceKind,
    pub abs: PathBuf,
    pub rel: PathBuf,
}

impl Eq for SourcePath {}

impl Hash for SourcePath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.abs.hash(state);
    }
}

impl PartialEq<Self> for SourcePath {
    fn eq(&self, other: &Self) -> bool {
        self.abs.eq(&other.abs)
    }
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

    /// Absolute path to source in filesystem
    pub fn abs(&self) -> &PathBuf {
        &self.abs
    }

    /// Relative path to source from project or build output root
    pub fn rel(&self) -> &PathBuf {
        &self.rel
    }

    pub fn _file_name(&self) -> String {
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
        let abs = build_dir.to_path().join(&rel);
        Self::new(SourceKind::FunctionBuild(build_dir), abs, rel)
    }

    pub fn to_relative_source(&self, path: &Path) -> Self {
        let project_dir = PathBuf::from(
            self.abs
                .to_string_lossy()
                .strip_suffix(&self.rel.to_string_lossy().to_string())
                .unwrap(),
        );
        let abs_path = match self.abs.file_name() {
            None => &self.abs,
            Some(_) => self.abs.parent().unwrap(),
        }
        .join(path);
        SourcePath::from_abs(
            &project_dir,
            match rewrite_current_and_parent_in_path(&abs_path) {
                Ok(maybe_rewritten) => maybe_rewritten.unwrap_or(abs_path),
                Err(err) => {
                    panic!(
                        "error collapsing parent path segments in path {}: {err}",
                        abs_path.to_string_lossy()
                    )
                }
            },
        )
    }
}

/// Removes `.` and `..` segments from paths, rewriting `..` to the parent directory
/// This fn returns Ok(None) if it's a noop (path does not have any current or parent segments)
pub fn rewrite_current_and_parent_in_path(p: &Path) -> Result<Option<PathBuf>, anyhow::Error> {
    let mut stack: Vec<String> = Vec::new();
    let mut changed = false;
    for path_component_os_str in p {
        let path_component = path_component_os_str.to_string_lossy();
        match path_component.as_ref() {
            "." => changed = true,
            ".." => {
                stack.pop();
                changed = true;
            }
            _ => stack.push(path_component.to_string()),
        }
    }
    if changed {
        Ok(Some(PathBuf::from(stack.join(MAIN_SEPARATOR_STR))))
    } else {
        Ok(None)
    }
}
