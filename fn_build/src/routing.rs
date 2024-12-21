use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FnRouting {
    HttpRoute(HttpRoute),
    Unsupported,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Delete,
    Patch,
    Post,
    Put,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HttpRoute {
    pub method: HttpMethod,
    pub path: String,
}

impl FnRouting {
    pub fn from_handler_fn(path: &Path, handler_fn_name: &str) -> Self {
        debug_assert!(path.is_relative());
        match HttpRoute::from_handler_fn(path, handler_fn_name) {
            None => Self::Unsupported,
            Some(http_route) => FnRouting::HttpRoute(http_route),
        }
    }
}

impl HttpRoute {
    pub fn from_handler_fn(path: &Path, handler_fn_name: &str) -> Option<Self> {
        debug_assert!(path.is_relative());
        match Self::extract_http_path(path) {
            None => None,
            Some(path) => match HttpMethod::try_from(handler_fn_name) {
                Err(_) => None,
                Ok(method) => Some(Self { method, path }),
            },
        }
    }

    fn extract_http_path(path: &Path) -> Option<String> {
        debug_assert!(path.is_relative());
        let mut parts: Vec<String> = Vec::new();
        for p in path.parent()?.components().rev() {
            if p.as_os_str().to_string_lossy().as_ref() == "routes" {
                return Some(parts.join("/"));
            } else {
                parts.insert(0, p.as_os_str().to_string_lossy().to_string());
            }
        }
        None
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
