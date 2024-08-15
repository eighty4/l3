use std::collections::HashMap;
use std::path::Path;

use crate::code::source::path::{SourceKind, SourcePath};
use crate::lambda::HttpMethod;

pub(crate) mod path;
pub(crate) mod tracker;
pub(crate) mod tree;
pub(crate) mod watcher;

#[cfg(test)]
mod path_test;
#[cfg(test)]
mod tree_test;
#[cfg(test)]
mod watcher_test;

#[derive(Clone)]
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
        debug_assert!(matches!(path.kind, SourceKind::OriginalSource));
        Self {
            imports,
            exported_fns,
            language,
            path,
        }
    }

    pub fn collect_http_handler_fn_names(&self) -> HashMap<HttpMethod, String> {
        let mut handler_fns = HashMap::new();
        for exported_fn in &self.exported_fns {
            if let Ok(http_method) = HttpMethod::try_from(exported_fn.as_str()) {
                if handler_fns.contains_key(&http_method) {
                    todo!("send LambdaNotification");
                    // return Err(anyhow!(
                    //     "multiple {http_method} functions found in source file {}",
                    //     self.path.rel.file_name().unwrap().to_string_lossy()
                    // ));
                }
                handler_fns.insert(http_method, exported_fn.clone());
            }
        }
        handler_fns
    }
}
