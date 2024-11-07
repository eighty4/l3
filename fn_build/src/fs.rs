use std::fs::{copy, create_dir_all, read_dir};
use std::io;
use std::path::Path;

pub fn copy_dir_all(from_dir: &Path, to: &Path) -> io::Result<()> {
    debug_assert!(from_dir.is_dir());
    create_dir_all(to)?;
    for dir_entry_result in read_dir(from_dir)? {
        let dir_entry = dir_entry_result?;
        let from = dir_entry.path();
        if from.is_dir() {
            copy_dir_all(&from, &to.join(from.file_name().unwrap()))?;
        } else {
            copy(&from, to.join(from.file_name().unwrap()))?;
        }
    }
    Ok(())
}
