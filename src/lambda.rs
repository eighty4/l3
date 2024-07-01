use std::fmt;
use std::path::PathBuf;

use anyhow::anyhow;

use crate::code::env::EnvVarSources;
use crate::code::source::SourceFile;

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
    pub route_key: RouteKey,
    pub source_file: SourceFile,
}

impl LambdaFn {
    pub fn new(
        env_var_sources: EnvVarSources,
        handler_fn: String,
        project_name: &String,
        route_key: RouteKey,
        source_file: SourceFile,
    ) -> Self {
        Self {
            env_var_sources,
            fn_name: create_fn_name(project_name, &route_key),
            handler_fn,
            route_key,
            source_file,
        }
    }

    pub fn handler_path(&self) -> String {
        format!(
            "routes/{}/{}.{}",
            self.route_key.http_path,
            self.source_file
                .file_name
                .strip_suffix(&self.source_file.extension)
                .unwrap()
                .strip_suffix('.')
                .unwrap(),
            self.handler_fn,
        )
    }
}
