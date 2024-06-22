use std::fs;
use std::path::PathBuf;

use anyhow::anyhow;

use crate::code::sha256::make_checksum;

pub trait ChecksumCached {
    #[allow(unused)]
    fn do_checksums_match(&self, api_id: &String) -> Result<bool, anyhow::Error> {
        Ok(match self.get_cached_checksum(api_id)? {
            None => false,
            Some(cached_checksum) => {
                cached_checksum.as_str() == self.get_source_checksum()?.as_str()
            }
        })
    }

    #[allow(unused)]
    fn get_cached_checksum(&self, api_id: &String) -> Result<Option<String>, anyhow::Error> {
        let checksum_path = self
            .get_project_dir()
            .join(self.get_relative_cached_checksum_path(api_id));
        if checksum_path.is_file() {
            Ok(Some(fs::read_to_string(checksum_path)?.trim().to_string()))
        } else {
            Ok(None)
        }
    }

    fn get_project_dir(&self) -> &PathBuf;

    fn get_relative_cached_checksum_path(&self, api_id: &String) -> PathBuf {
        let source_path = self.get_relative_source_path();
        PathBuf::from(".l3")
            .join(api_id)
            .join(source_path.with_file_name(format!(
                "{}.sha256",
                source_path.file_name().unwrap().to_string_lossy()
            )))
    }

    fn get_relative_source_path(&self) -> &PathBuf;

    fn get_source_checksum(&self) -> Result<String, anyhow::Error> {
        let source_path = self.get_project_dir().join(self.get_relative_source_path());
        if source_path.is_file() {
            make_checksum(&source_path)
        } else {
            Err(anyhow!(
                "unable to find file to generate checksum for {}",
                source_path.to_string_lossy()
            ))
        }
    }

    fn update_checksum_cache(&self, api_id: &String) -> Result<(), anyhow::Error> {
        let checksum_rel_path = self.get_relative_cached_checksum_path(api_id);
        let checksum_dir_abs_path = self
            .get_project_dir()
            .join(checksum_rel_path.parent().unwrap());
        if !checksum_dir_abs_path.exists() {
            fs::create_dir_all(&checksum_dir_abs_path)?;
        }
        let checksum_abs_path = checksum_dir_abs_path.join(checksum_rel_path.file_name().unwrap());
        match fs::write(&checksum_abs_path, self.get_source_checksum()?) {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!(
                "failed writing checksum {} for error: {}",
                &checksum_abs_path.to_string_lossy(),
                err.to_string()
            )),
        }
    }
}
