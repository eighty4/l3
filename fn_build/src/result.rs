use std::path::PathBuf;

#[derive(Clone)]
pub enum ModuleImport {
    #[allow(unused)]
    PackageDependency {
        package: String,
        subpath: Option<String>,
    },
    #[allow(unused)]
    RelativeSource(PathBuf),
    #[allow(unused)]
    Unknown(String),
}

#[derive(Clone)]
pub struct FnSource {
    pub imports: Vec<ModuleImport>,
    pub path: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum FnBuildError {
    #[error("entrypoint file type is unsupported")]
    InvalidFileType,
}

pub type FnBuildResult<T> = Result<T, FnBuildError>;
