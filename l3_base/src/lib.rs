use std::fs::read_dir;
use std::path::{Path, PathBuf};

/// Paths returned are relative if dir is relative and absolute if dir is absolute
pub fn collect_files(dir: &Path) -> Vec<PathBuf> {
    assert!(dir.is_dir());
    let mut paths = Vec::new();
    for dir_entry_result in read_dir(dir).unwrap() {
        let dir_entry = dir_entry_result.unwrap();
        let p = dir_entry.path();
        if p.is_dir() {
            paths.append(&mut collect_files(&p));
        } else {
            paths.push(p);
        }
    }
    paths
}
