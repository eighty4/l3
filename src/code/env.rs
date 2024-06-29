use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use lazy_static::lazy_static;
use regex::Regex;

use crate::code::checksum::ChecksumCached;
use crate::code::source::SourceFile;
use crate::lambda::{HttpMethod, RouteKey};

lazy_static! {
    static ref ENV_FILENAME_REGEX: Regex =
        Regex::new(r"^(?<http_method>delete|get|patch|post|put)?\.env$")
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
    method_env_file: Option<EnvFile>,
    path_env_file: Option<EnvFile>,
    route_key: RouteKey,
}

impl EnvVarSources {
    pub fn new(env_files: Vec<EnvFile>, route_key: RouteKey) -> Result<Self, anyhow::Error> {
        debug_assert!(env_files.len() <= 3);
        let mut method_env_file: Option<EnvFile> = None;
        let mut path_env_file: Option<EnvFile> = None;
        for env_file in env_files {
            if env_file.http_method.is_some() {
                method_env_file = Some(env_file);
            } else {
                path_env_file = Some(env_file);
            }
        }
        Ok(Self {
            method_env_file,
            path_env_file,
            route_key,
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

    // todo syncing env vars is buggy af
    pub fn requires_update(&self, api_id: &String) -> Result<bool, anyhow::Error> {
        Ok(self.method_env_requires_update(api_id)? || self.path_env_requires_update(api_id)?)
    }

    fn method_env_requires_update(&self, api_id: &String) -> Result<bool, anyhow::Error> {
        let requires_update = match &self.method_env_file {
            None => self.method_env_file_path().is_file(),
            Some(env_file) => !env_file.source_file.do_checksums_match(api_id)?,
        };
        Ok(requires_update)
    }

    fn path_env_requires_update(&self, api_id: &String) -> Result<bool, anyhow::Error> {
        let requires_update = match &self.path_env_file {
            None => self.path_env_file_path().is_file(),
            Some(env_file) => !env_file.source_file.do_checksums_match(api_id)?,
        };
        Ok(requires_update)
    }

    fn method_env_file_path(&self) -> PathBuf {
        self.path_env_file_path()
            .join(format!("{}.env", self.route_key.http_method))
    }

    fn path_env_file_path(&self) -> PathBuf {
        self.route_key.to_route_dir_path().join(".env")
    }

    pub fn update_cached_checksums(&self, api_id: &String) -> Result<(), anyhow::Error> {
        let api_data_dir = PathBuf::from(".l3").join(api_id);
        for (checksum_path, env_file) in &[
            (
                api_data_dir.join(self.method_env_file_path()),
                &self.method_env_file,
            ),
            (
                api_data_dir.join(self.path_env_file_path()),
                &self.path_env_file,
            ),
        ] {
            match env_file {
                None => {
                    if checksum_path.is_file() {
                        let _ = fs::remove_file(checksum_path);
                    }
                }
                Some(env_file) => {
                    let _ = env_file.source_file.update_checksum_cache(api_id);
                }
            };
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct EnvFile {
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
    pub fn is_for_http_method(&self, http_method: &HttpMethod) -> bool {
        match self.http_method.as_ref() {
            None => true,
            Some(hm) => hm == http_method,
        }
    }

    pub fn read_env_variables(&self) -> Result<HashMap<String, String>, anyhow::Error> {
        let contents = fs::read_to_string(&self.source_file.path)?;
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
