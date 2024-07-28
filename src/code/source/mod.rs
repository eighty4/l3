use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::anyhow;

use crate::code::build::BuildMode;
use crate::code::source::path::{SourceKind, SourcePath};
use crate::lambda::HttpMethod;

pub(crate) mod path;
pub(crate) mod tracker;
pub(crate) mod tree;
pub(crate) mod watcher;

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
    #[allow(unused)]
    PackageDependency {
        package: String,
        subpath: Option<String>,
    },
    #[allow(unused)]
    RelativeSource(SourcePath),
    #[allow(unused)]
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

    pub fn collect_handler_fn_names(&self) -> Result<HashMap<HttpMethod, String>, anyhow::Error> {
        let mut handler_fns = HashMap::new();
        for exported_fn in &self.exported_fns {
            if let Ok(http_method) = HttpMethod::try_from(exported_fn.as_str()) {
                if handler_fns.contains_key(&http_method) {
                    return Err(anyhow!(
                        "multiple {http_method} functions found in source file {}",
                        self.path.rel.file_name().unwrap().to_string_lossy()
                    ));
                }
                handler_fns.insert(http_method, exported_fn.clone());
            }
        }
        Ok(handler_fns)
    }
}
