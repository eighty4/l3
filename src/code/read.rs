use std::fs;
use std::path::PathBuf;

/// Paths are relative
pub fn recursively_read_dirs(dir_path: &PathBuf) -> Result<Vec<PathBuf>, anyhow::Error> {
    let mut files: Vec<PathBuf> = Vec::new();
    for dir_entry_result in fs::read_dir(dir_path)? {
        let dir_entry = dir_entry_result?;
        let path = dir_entry.path();
        if path.is_dir() {
            files.append(&mut recursively_read_dirs(&path)?);
        } else {
            files.push(path);
        }
    }
    Ok(files)
}
