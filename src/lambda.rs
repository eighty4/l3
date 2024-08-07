use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::anyhow;

use crate::code::env::EnvVarSources;

fn create_fn_name(project_name: &String, route_key: &RouteKey) -> String {
    format!(
        "l3-{}-{}-{}",
        project_name,
        route_key.http_path.replace('/', "-"),
        route_key.http_method.to_string().to_lowercase(),
    )
}

// todo trait for lambdas with to_fn_name and to_dir_path not strictly for http lambdas
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct RouteKey {
    pub http_method: HttpMethod,
    pub http_path: String,
}

impl RouteKey {
    pub fn extract_http_path(path: &Path) -> Option<String> {
        debug_assert!(path.is_relative());
        let mut parts = Vec::new();
        for p in path.parent().unwrap().components().rev() {
            if p.as_os_str().to_string_lossy().as_ref() == "routes" {
                return Some(PathBuf::from_iter(parts).to_string_lossy().to_string());
            } else {
                parts.insert(0, p);
            }
        }
        None
    }

    pub fn new(http_method: HttpMethod, http_path: String) -> Self {
        Self {
            http_method,
            http_path: if let Some(s) = http_path.strip_prefix('/') {
                s.to_string()
            } else {
                http_path
            },
        }
    }

    pub fn to_fn_name(&self, project_name: &String) -> String {
        create_fn_name(project_name, self)
    }

    pub fn to_route_dir_path(&self) -> PathBuf {
        PathBuf::from("routes").join(&self.http_path)
    }

    pub fn to_route_key_string(&self) -> String {
        format!("{} /{}", self.http_method, self.http_path)
    }
}

impl TryFrom<String> for RouteKey {
    type Error = anyhow::Error;

    fn try_from(route_key_str: String) -> Result<Self, Self::Error> {
        let (http_method_str, http_path_str) = route_key_str.split_once(' ').unwrap();
        let http_method = HttpMethod::try_from(http_method_str)?;
        let http_path = http_path_str.trim().to_string();
        Ok(RouteKey::new(http_method, http_path))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HttpMethod {
    Delete,
    Get,
    Patch,
    Post,
    Put,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
        }
    }
}

impl<'a> TryFrom<&'a str> for HttpMethod {
    type Error = anyhow::Error;

    fn try_from(http_method_str: &'a str) -> Result<Self, Self::Error> {
        let http_method = match http_method_str.to_uppercase().as_str() {
            "DELETE" => Some(HttpMethod::Delete),
            "GET" => Some(HttpMethod::Get),
            "PATCH" => Some(HttpMethod::Patch),
            "POST" => Some(HttpMethod::Post),
            "PUT" => Some(HttpMethod::Put),
            _ => None,
        };
        http_method.ok_or(anyhow!(
            "could not resolve http method from {http_method_str}"
        ))
    }
}

#[derive(Clone)]
pub struct LambdaFn {
    pub env_var_sources: EnvVarSources,
    pub fn_name: String,
    pub handler_fn: String,
    pub path: PathBuf,
    pub route_key: RouteKey,
}

impl LambdaFn {
    pub fn new(
        env_var_sources: EnvVarSources,
        handler_fn: String,
        path: PathBuf,
        project_name: &String,
        route_key: RouteKey,
    ) -> Self {
        Self {
            env_var_sources,
            fn_name: create_fn_name(project_name, &route_key),
            handler_fn,
            path,
            route_key,
        }
    }

    pub fn handler_path(&self) -> String {
        let file_name = self.path.file_name().unwrap().to_string_lossy();
        let extension = self.path.extension().unwrap().to_string_lossy();
        format!(
            "routes/{}/{}.{}",
            self.route_key.http_path,
            file_name
                .strip_suffix(extension.as_ref())
                .unwrap()
                .strip_suffix('.')
                .unwrap(),
            self.handler_fn,
        )
    }
}
