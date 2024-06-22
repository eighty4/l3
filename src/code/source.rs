use std::path::PathBuf;

use crate::code::checksum::ChecksumCached;
use crate::code::sha256::make_checksum;

#[derive(Clone)]
pub struct SourceFile {
    pub extension: String,
    pub file_name: String,
    pub hash: String,
    pub path: PathBuf,
    project_dir: PathBuf,
}

impl SourceFile {
    pub fn create(path: PathBuf, project_dir: PathBuf) -> Result<Self, anyhow::Error> {
        assert!(path.is_relative());
        let abs_path = &project_dir.join(&path);
        assert!(abs_path.is_file());
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let extension = if file_name.as_str() == ".env" {
            file_name.strip_prefix('.').unwrap().to_string()
        } else {
            path.extension().unwrap().to_string_lossy().to_string()
        };
        Ok(Self {
            extension,
            file_name,
            hash: make_checksum(abs_path)?,
            path,
            project_dir,
        })
    }
}

impl ChecksumCached for SourceFile {
    fn get_project_dir(&self) -> &PathBuf {
        &self.project_dir
    }

    fn get_relative_source_path(&self) -> &PathBuf {
        &self.path
    }

    fn get_source_checksum(&self) -> Result<String, anyhow::Error> {
        Ok(self.hash.clone())
    }
}
