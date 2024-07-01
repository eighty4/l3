use std::path::PathBuf;

#[derive(Clone)]
pub struct SourceFile {
    pub extension: String,
    pub file_name: String,
    /// Relative path
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
            path,
            project_dir,
        })
    }

    pub fn abs_path(&self) -> PathBuf {
        self.project_dir.join(&self.path)
    }
}
