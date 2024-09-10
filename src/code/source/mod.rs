use crate::code::env::EnvVarSources;
use crate::code::source::path::{SourceKind, SourcePath};
use crate::lambda::{HttpMethod, LambdaFn, RouteKey};
use crate::project::Lx3ProjectDeets;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

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

#[derive(Clone, PartialEq)]
pub enum Language {
    JavaScript,
    TypeScript,
    Python,
}

impl Language {
    pub fn from_extension(path: &Path) -> Option<Language> {
        match path.extension()?.to_string_lossy().as_ref() {
            "js" | "mjs" => Some(Language::JavaScript),
            "py" => Some(Language::Python),
            "ts" => Some(Language::TypeScript),
            &_ => None,
        }
    }
}

pub enum ModuleImports {
    Empty,
    Processed(Vec<ModuleImport>),
    Unprocessed(Vec<String>),
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
    pub imports: ModuleImports,
    pub language: Language,
    pub path: SourcePath,
}

impl SourceFile {
    pub fn new(
        exported_fns: Vec<String>,
        imports: ModuleImports,
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
                handler_fns.insert(http_method, exported_fn.clone());
                // todo send LambdaNotification src warning if multiple cased fn names
                //  for the same handler fn like GET and get
                //   if handler_fns.contains_key(&http_method) {}
            }
        }
        handler_fns
    }

    pub fn collect_lambda_fns(
        &self,
        project_deets: &Arc<Lx3ProjectDeets>,
    ) -> Option<Vec<LambdaFn>> {
        debug_assert!(self.path.rel.starts_with("routes"));
        let handler_fns = self.collect_http_handler_fn_names();
        if handler_fns.is_empty() {
            None
        } else {
            let mut lambda_fns: Vec<LambdaFn> = Vec::new();
            let http_path = RouteKey::extract_http_path(&self.path.rel).unwrap();
            for (http_method, handler_fn) in handler_fns {
                let route_key = RouteKey::new(http_method, http_path.clone());
                let env_var_sources =
                    EnvVarSources::new(&project_deets.project_dir, &route_key).unwrap();
                lambda_fns.push(LambdaFn::new(
                    env_var_sources,
                    handler_fn,
                    self.path.clone(),
                    project_deets.clone(),
                    route_key,
                ));
            }
            Some(lambda_fns)
        }
    }
}
