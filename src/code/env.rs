use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use regex::Regex;

use crate::code::source::SourceFile;
use crate::lambda::{HttpMethod, RouteKey};

lazy_static! {
    static ref ENV_FILENAME_REGEX: Regex =
        Regex::new(r"^lambda((?:\.)(?<http_method>delete|get|patch|post|put))?\.env$")
            .expect("env filename regex");
}

pub fn is_valid_env_file_name(file_name: &str) -> bool {
    ENV_FILENAME_REGEX.is_match(file_name)
}

pub fn parse_env_file_name_for_http_method(file_name: &str) -> Option<HttpMethod> {
    debug_assert!(is_valid_env_file_name(file_name));
    ENV_FILENAME_REGEX
        .captures(file_name)
        .unwrap()
        .name("http_method")
        .map(|m| HttpMethod::try_from(m.as_str()).unwrap())
}

#[derive(Clone)]
pub struct EnvVarSources {
    pub method_env_file: Option<EnvFile>,
    /// Relative path
    pub method_env_file_path: PathBuf,
    pub path_env_file: Option<EnvFile>,
    /// Relative path
    pub path_env_file_path: PathBuf,
}

impl EnvVarSources {
    pub fn new(project_dir: &Path, route_key: &RouteKey) -> Result<Self, anyhow::Error> {
        let lambda_dir_path = route_key.to_route_dir_path();
        let method_env_file_path = lambda_dir_path.join(format!(
            "lambda.{}.env",
            route_key.http_method.to_string().to_lowercase()
        ));
        let method_env_file = if project_dir.join(&method_env_file_path).exists() {
            Some(EnvFile::create(
                method_env_file_path.clone(),
                project_dir.to_path_buf(),
            )?)
        } else {
            None
        };
        let path_env_file_path = lambda_dir_path.join("lambda.env");
        let path_env_file = if project_dir.join(&path_env_file_path).exists() {
            Some(EnvFile::create(
                path_env_file_path.clone(),
                project_dir.to_path_buf(),
            )?)
        } else {
            None
        };
        Ok(Self {
            method_env_file,
            method_env_file_path,
            path_env_file,
            path_env_file_path,
        })
    }

    pub fn read_env_variables(&self) -> Result<Option<HashMap<String, String>>, anyhow::Error> {
        if self.method_env_file.is_none() && self.path_env_file.is_none() {
            Ok(None)
        } else {
            let mut env_vars = HashMap::new();
            if let Some(env_file) = &self.path_env_file {
                for (k, v) in env_file.read_env_variables()? {
                    env_vars.insert(k, v);
                }
            }
            if let Some(env_file) = &self.method_env_file {
                for (k, v) in env_file.read_env_variables()? {
                    env_vars.insert(k, v);
                }
            }
            Ok(Some(env_vars))
        }
    }

    #[allow(unused)]
    pub fn source_paths(&self) -> Vec<PathBuf> {
        [&self.method_env_file, &self.path_env_file]
            .iter()
            .filter(|opt| opt.is_some())
            .map(|opt| {
                opt.as_ref()
                    .map(|env_file| env_file.source_file.path.clone())
                    .unwrap()
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct EnvFile {
    #[allow(unused)]
    pub http_method: Option<HttpMethod>,
    pub source_file: SourceFile,
}

impl EnvFile {
    pub fn create(path: PathBuf, project_dir: PathBuf) -> Result<Self, anyhow::Error> {
        assert!(path.is_relative());
        let source_file = SourceFile::create(path, project_dir)?;
        let http_method = parse_env_file_name_for_http_method(source_file.file_name.as_str());
        Ok(Self {
            http_method,
            source_file,
        })
    }
}

impl EnvFile {
    #[allow(unused)]
    pub fn is_for_http_method(&self, http_method: &HttpMethod) -> bool {
        match self.http_method.as_ref() {
            None => true,
            Some(hm) => hm == http_method,
        }
    }

    pub fn read_env_variables(&self) -> Result<HashMap<String, String>, anyhow::Error> {
        let contents = fs::read_to_string(self.source_file.abs_path())?;
        let mut vars: HashMap<String, String> = HashMap::new();
        for line in contents.lines() {
            let mut parts = line.splitn(2, '=');
            let key = parts.next().unwrap().to_string();
            let val = parts.next().unwrap().to_string();
            vars.insert(key, val);
        }
        Ok(vars)
    }
}
