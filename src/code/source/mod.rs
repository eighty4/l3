use std::path::{Path, PathBuf};

use crate::code::build::BuildMode;
use crate::code::source::path::{SourceKind, SourcePath};

pub(crate) mod path;
pub(crate) mod tracker;
pub(crate) mod tree;

#[cfg(test)]
mod path_test;
#[cfg(test)]
mod tracker_test;

#[derive(Clone, PartialEq)]
pub enum Language {
    JavaScript,
    TypeScript,
    Python,
}

impl Language {
    pub fn from_extension(path: &Path) -> Option<Language> {
        match path.extension().unwrap().to_string_lossy().as_ref() {
            "js" | "mjs" => Some(Language::JavaScript),
            "py" => Some(Language::Python),
            "ts" => Some(Language::TypeScript),
            &_ => None,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct FunctionBuildDir {
    api_id: String,
    build_mode: BuildMode,
    fn_name: String,
}

impl FunctionBuildDir {
    pub fn new(api_id: String, build_mode: BuildMode, fn_name: String) -> Self {
        Self {
            api_id,
            build_mode,
            fn_name,
        }
    }

    pub fn abs(&self, project_dir: &Path) -> PathBuf {
        project_dir.join(self.rel())
    }

    pub fn rel(&self) -> PathBuf {
        PathBuf::from(".l3")
            .join(&self.api_id)
            .join(&self.fn_name)
            .join(match self.build_mode {
                BuildMode::Debug => "dev",
                BuildMode::Release => "prod",
            })
    }
}

pub enum ModuleImport {
    // https://nodejs.org/api/packages.html#subpath-imports
    #[allow(unused)]
    NodeSubpathImport {
        declared: String,
        path: SourcePath,
    },
    PackageDependency {
        package: String,
        subpath: Option<String>,
    },
    RelativeSource(SourcePath),
    // https://www.typescriptlang.org/tsconfig/#paths
    // https://www.typescriptlang.org/docs/handbook/modules/reference.html#paths
    #[allow(unused)]
    TypeScriptPathMapping {
        declared: String,
        path: SourcePath,
    },
    Unknown(String),
}

pub struct SourceFile {
    pub exported_fns: Vec<String>,
    #[allow(unused)]
    pub imports: Vec<ModuleImport>,
    #[allow(unused)]
    pub language: Language,
    pub path: SourcePath,
}

impl SourceFile {
    pub fn new(
        exported_fns: Vec<String>,
        imports: Vec<ModuleImport>,
        language: Language,
        path: SourcePath,
    ) -> Self {
        debug_assert!(path.kind == SourceKind::OriginalSource);
        Self {
            imports,
            exported_fns,
            language,
            path,
        }
    }
}
