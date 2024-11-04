use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::code::env::EnvVarSources;
use crate::code::read::recursively_read_dirs;
use crate::code::sha256::make_checksum;
use crate::code::source::path::{SourceKind, SourcePath};

// todo move checksum tree instances into source tree or lambda fn
/// Manages checksum diffing for sources packaged with a Lambda Function
pub struct ChecksumTree {
    /// Map of relative source paths to sha256 hash
    checksums: HashMap<PathBuf, String>,
    /// Absolute path to {project_dir}/.l3/{api_id}/{fn_name}/checksums
    checksum_dir: PathBuf,
    /// Absolute path to project
    project_dir: PathBuf,
}

impl ChecksumTree {
    pub fn dir_path(project_dir: &Path, api_id: &String, fn_name: &String) -> PathBuf {
        project_dir
            .join(".l3")
            .join("aws")
            .join(api_id)
            .join(fn_name)
            .join("checksums")
    }

    pub async fn new(
        project_dir: PathBuf,
        api_id: &String,
        fn_name: &String,
    ) -> Result<Self, anyhow::Error> {
        let mut checksums = HashMap::new();
        let checksum_dir = Self::dir_path(&project_dir, api_id, fn_name);
        if checksum_dir.is_dir() {
            for abs_path in recursively_read_dirs(&checksum_dir).await? {
                checksums.insert(
                    PathBuf::from(&abs_path.strip_prefix(&checksum_dir)?),
                    fs::read_to_string(&abs_path)?,
                );
            }
        } else {
            fs::create_dir_all(&checksum_dir)?;
        }
        Ok(ChecksumTree {
            checksum_dir,
            checksums,
            project_dir,
        })
    }

    pub fn do_checksums_match(&self, path: &PathBuf) -> Result<bool, anyhow::Error> {
        debug_assert!(path.is_relative());
        let made = make_checksum(&self.project_dir.join(path))?;
        let matches = match self.checksums.get(path) {
            None => false,
            Some(cached) => cached.as_str() == made.as_str(),
        };
        Ok(matches)
    }

    #[allow(unused)]
    pub fn do_all_checksums_match(&self, paths: &Vec<PathBuf>) -> Result<bool, anyhow::Error> {
        debug_assert!(paths.iter().all(|p| p.is_relative()));
        for p in paths {
            if !self.do_checksums_match(p)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn do_env_checksums_match(
        &self,
        deployed_env_vars: &Option<HashMap<String, String>>,
        env_var_sources: &EnvVarSources,
    ) -> Result<bool, anyhow::Error> {
        let has_method_env_file = env_var_sources.method_env_file.is_some();
        if !has_method_env_file {
            if self
                .checksums
                .contains_key(&env_var_sources.method_env_file_path)
            {
                return Ok(false);
            }
        } else if !self.do_checksums_match(&env_var_sources.method_env_file_path)? {
            return Ok(false);
        }
        let has_path_env_file = env_var_sources.path_env_file.is_some();
        if !has_path_env_file {
            if self
                .checksums
                .contains_key(&env_var_sources.path_env_file_path)
            {
                return Ok(false);
            }
        } else if !self.do_checksums_match(&env_var_sources.path_env_file_path)? {
            return Ok(false);
        }
        // only matching checksums if an env file exists or the deployed fn has no env vars
        Ok(has_method_env_file
            || has_path_env_file
            || deployed_env_vars
                .as_ref()
                .map_or(true, |env| env.is_empty()))
    }

    pub fn remove_checksum(&mut self, path: &PathBuf) {
        debug_assert!(path.is_relative());
        let _ = fs::remove_file(self.checksum_dir.join(path));
        self.checksums.remove(path);
    }

    pub fn update_checksum(&mut self, path: PathBuf) -> Result<(), anyhow::Error> {
        debug_assert!(path.is_relative());
        let checksum_abs_path = self.checksum_dir.join(&path);
        if let Some(checksum_parent_dir) = checksum_abs_path.parent() {
            if !checksum_parent_dir.is_dir() {
                fs::create_dir_all(checksum_parent_dir)?;
            }
        }
        let hash = make_checksum(&self.project_dir.join(&path))?;
        fs::write(checksum_abs_path, &hash)?;
        self.checksums.insert(path, hash);
        Ok(())
    }

    pub fn update_all_checksums(&mut self, paths: &Vec<SourcePath>) -> Result<(), anyhow::Error> {
        for p in paths {
            debug_assert!(matches!(p.kind, SourceKind::OriginalSource));
            self.update_checksum(p.rel().clone())?;
        }
        Ok(())
    }

    pub fn update_env_var_checksums(
        &mut self,
        env_var_sources: &EnvVarSources,
    ) -> Result<(), anyhow::Error> {
        // todo impl iter for EnvVarSources
        if env_var_sources.method_env_file.is_none() {
            self.remove_checksum(&env_var_sources.method_env_file_path);
        } else {
            self.update_checksum(env_var_sources.method_env_file_path.clone())?;
        }
        if env_var_sources.path_env_file.is_none() {
            self.remove_checksum(&env_var_sources.path_env_file_path);
        } else {
            self.update_checksum(env_var_sources.path_env_file_path.clone())?;
        }
        Ok(())
    }
}
